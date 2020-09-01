use crate::{
    collections::XorHashMap,
    gfx::{AnimatedMesh, AnimatedVertex},
    math::{Vector2, Vector3},
    util::{self, BoxedError},
};
use std::{io::Read, marker::PhantomData};
use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader, ParserConfig};

#[derive(Debug, Copy, Clone)]
enum State {
    Init,
    ColladaTag,
    UnimplementedTagLevel,
    Libraries,
    GeometryLibraryChild,
    GeometryChild,
    MeshChild,
    SourceChild,
    SourceFloatArrayText,
    VerticesChild,
    TrianglesChild,
    TrianglesPrimitiveText,
}

#[derive(Debug)]
enum SourceKind {
    FloatArray(Vec<f32>),
}

#[derive(Debug, Default)]
struct Source {
    pub kind: Option<SourceKind>,
}

#[derive(Debug)]
enum TriangleInputKind {
    Vertex,
    Normal,
    TexCoord,
}

#[derive(Debug)]
struct TriangleInput {
    pub kind: TriangleInputKind,
    pub offset: usize,
}

/// A quick and dirty collada parser
///
/// Long term this will be used to make a converter to a custom format.
///
/// # Limitations
/// - Single geometry with single mesh
#[derive(Debug, Default)]
pub struct ColladaReader<R: Read> {
    // Parser state data
    states: Vec<State>,

    // Data source blobs
    sources: XorHashMap<String, Source>,

    // Map of vertices id to source id of vertex data. No idea why collada makes this indirect
    vertices_mapping: XorHashMap<String, String>,

    // Inputs to construct mesh triangles. Key is source or vertex mapping id.
    triangle_inputs: XorHashMap<String, TriangleInput>,

    // The id of the thing we're inside.. For context.
    latest_id: String,

    // Buffer to store primitive indices.
    // unfortunately collada indices are like OBJ indices where there are not shared per vertex.
    // So we have to use these indices to compute new shared indices.
    primitive_indices: Vec<usize>,

    // Buffer to store positions
    positions: Vec<Vector3>,

    // Buffer for store normals
    normals: Vec<Vector3>,

    // Buffer for store tex coords
    tex_coords: Vec<Vector2>,

    // TODO: I wanted to re-use the xml reader to reduce new allocations,
    //   but xml-rs does not support this. For now we'll pretend it
    //   does in the API. In the future we'd store the reader here...
    //   In fact we've gone crazy with allocations. Would be nice to reduce them...
    //   Long term. Make this into a tool that converts collada to a custom binary format.
    phantom: PhantomData<R>,
}

impl<R: Read> ColladaReader<R> {
    pub fn read_into(&mut self, reader: &mut R, mesh: &mut AnimatedMesh) -> Result<(), BoxedError> {
        mesh.clear();

        self.states.clear();
        self.sources.clear();
        self.vertices_mapping.clear();
        self.triangle_inputs.clear();
        self.latest_id.clear();
        self.primitive_indices.clear();
        self.positions.clear();
        self.normals.clear();
        self.push(State::Init);

        let mut xml_reader = EventReader::new_with_config(
            reader,
            ParserConfig {
                // Disable whitespace events and ignore stuff we dont care about
                trim_whitespace: true,
                whitespace_to_characters: true,
                cdata_to_characters: true,
                ..ParserConfig::default()
            },
        );

        // The ol' shifty-reduce-y
        loop {
            let state = self.top().ok_or("Parser in invalid state")?;
            let event = xml_reader.next()?;

            match state {
                State::Init => match event {
                    XmlEvent::StartDocument { .. } => {
                        self.push(State::ColladaTag);
                    }
                    XmlEvent::EndDocument { .. } => {}
                    _ => unimplemented!("{:?}", event),
                },

                // If we are in a tag that we dont implement, we obviously
                // dont support its sub-tags.
                State::UnimplementedTagLevel => match event {
                    XmlEvent::StartElement { .. } => {
                        self.push(State::UnimplementedTagLevel);
                    }
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    XmlEvent::Characters(_) => {}
                    _ => unimplemented!("{:?}", event),
                },

                State::ColladaTag => match event {
                    XmlEvent::StartElement { name, .. } => {
                        if name.local_name != "COLLADA" {
                            return util::boxed_err("First tag is supposed to be COLLADA");
                        }
                        self.push(State::Libraries);
                    }
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    XmlEvent::EndDocument { .. } => break, // End of doc!
                    _ => unimplemented!("{:?}", event),
                },

                State::Libraries => match event {
                    XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                        "library_geometries" => {
                            self.push(State::GeometryLibraryChild);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::GeometryLibraryChild => match event {
                    XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                        "geometry" => {
                            self.push(State::GeometryChild);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::GeometryChild => match event {
                    XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                        "mesh" => {
                            self.push(State::MeshChild);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::MeshChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "source" => {
                            let id = Self::find_attribute(&attributes, "id")
                                .ok_or("Mesh sources must have ids")?;
                            self.set_latest_id(&id.value);
                            self.sources.insert(id.value.clone(), Source::default());
                            self.push(State::SourceChild);
                        }
                        "vertices" => {
                            let id = Self::find_attribute(&attributes, "id")
                                .ok_or("Mesh vertices must have ids")?;
                            self.set_latest_id(&id.value);
                            self.push(State::VerticesChild);
                        }
                        "triangles" => self.push(State::TrianglesChild),
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::SourceChild => match event {
                    XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                        "float_array" => {
                            self.push(State::SourceFloatArrayText);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::SourceFloatArrayText => match event {
                    XmlEvent::Characters(text) => {
                        let source = self
                            .sources
                            .get_mut(&self.latest_id)
                            .ok_or("Parser is in invalid state")?;
                        let mut floats = Vec::new();
                        for float in text.split_whitespace() {
                            floats.push(util::parse(float)?);
                        }
                        source.kind = Some(SourceKind::FloatArray(floats));
                    }
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::VerticesChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "input" => {
                            let source = Self::find_attribute(&attributes, "source")
                                .ok_or("Vertex inputs must have a source")?;
                            // Trim off the # since this is a ref link and save it
                            self.vertices_mapping
                                .insert(self.latest_id.clone(), Self::trim_ref(&source.value));
                            // We dont care about child nodes of input
                            self.push(State::UnimplementedTagLevel);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::TrianglesChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "input" => {
                            let semantic = Self::find_attribute(&attributes, "semantic")
                                .ok_or("Triangles inputs must have a semantic")?;
                            let source = Self::find_attribute(&attributes, "source")
                                .ok_or("Triangles inputs must have a source")?;
                            let offset = util::parse(
                                &Self::find_attribute(&attributes, "offset")
                                    .ok_or("Triangles inputs must have a offset")?
                                    .value,
                            )?;

                            // Just quickly check if the source is referring to something in
                            // the vertices map... And replace it
                            let mut source = Self::trim_ref(&source.value);
                            if let Some(mapping) = self.vertices_mapping.get(&source) {
                                source.clear();
                                source.push_str(mapping);
                            }

                            self.triangle_inputs.insert(
                                source.clone(),
                                match semantic.value.as_str() {
                                    "VERTEX" => TriangleInput {
                                        offset,
                                        kind: TriangleInputKind::Vertex,
                                    },
                                    "NORMAL" => TriangleInput {
                                        offset,
                                        kind: TriangleInputKind::Normal,
                                    },
                                    "TEXCOORD" => TriangleInput {
                                        offset,
                                        kind: TriangleInputKind::TexCoord,
                                    },
                                    i => unimplemented!("{:?}", i),
                                },
                            );
                            // We dont care about child nodes of input
                            self.push(State::UnimplementedTagLevel);
                        }
                        "p" => {
                            self.push(State::TrianglesPrimitiveText);
                        }
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::TrianglesPrimitiveText => match event {
                    XmlEvent::Characters(text) => {
                        self.primitive_indices.clear();
                        for index in text.split_whitespace() {
                            self.primitive_indices.push(util::parse(index)?);
                        }

                        let num_inputs = self.triangle_inputs.len();
                        // This is not optimal since we are re-iterating for every input type.
                        // But its fine for now.
                        for (id, input) in &self.triangle_inputs {
                            let source = self
                                .sources
                                .get(id)
                                .ok_or("Input source no longer exists")?;
                            let offset = input.offset;
                            // funky iterator...
                            for &index in self
                                .primitive_indices
                                .iter()
                                .skip(offset)
                                .step_by(num_inputs)
                            {
                                match &input.kind {
                                    TriangleInputKind::Vertex => match &source.kind {
                                        Some(SourceKind::FloatArray(positions)) => {
                                            self.positions.push(
                                                (
                                                    positions[index],
                                                    positions[index + 1],
                                                    positions[index + 2],
                                                )
                                                    .into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                    TriangleInputKind::Normal => match &source.kind {
                                        Some(SourceKind::FloatArray(normals)) => {
                                            self.normals.push(
                                                (
                                                    normals[index],
                                                    normals[index + 1],
                                                    normals[index + 2],
                                                )
                                                    .into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                    TriangleInputKind::TexCoord => match &source.kind {
                                        Some(SourceKind::FloatArray(tex_coords)) => {
                                            self.tex_coords.push(
                                                (tex_coords[index], tex_coords[index + 1]).into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                }
                            }
                        }
                    }
                    XmlEvent::EndElement { .. } => {
                        self.pop();
                    }
                    _ => unimplemented!("{:?}", event),
                },
            }
        }

        // TODO: Compress like indices
        for (i, ((&position, &normal), _)) in self
            .positions
            .iter()
            .zip(self.normals.iter())
            .zip(self.tex_coords.iter())
            .enumerate()
        {
            mesh.add_vertex(AnimatedVertex::new(position, normal));
            mesh.add_index(i as u32);
        }
        Ok(())
    }

    #[inline]
    fn push(&mut self, state: State) {
        self.states.push(state);
    }

    #[inline]
    fn top(&self) -> Option<State> {
        self.states.last().copied()
    }

    #[inline]
    fn pop(&mut self) -> Option<State> {
        self.states.pop()
    }

    fn find_attribute<'a>(
        attributes: &'a [OwnedAttribute],
        name: &str,
    ) -> Option<&'a OwnedAttribute> {
        attributes.iter().find(|attr| attr.name.local_name == name)
    }

    #[inline]
    fn set_latest_id(&mut self, value: &str) {
        self.latest_id.clear();
        self.latest_id.push_str(value);
    }

    #[inline]
    fn trim_ref(value: &str) -> String {
        value[1..].to_owned()
    }
}

#[cfg(test)]
mod test {
    use crate::gfx::{AnimatedMesh, ColladaReader};
    use std::io::Cursor;

    #[test]
    fn sanity_test() {
        let test = r##"
        <?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <asset>
    <contributor>
      <author>Blender User</author>
      <authoring_tool>Blender 2.82.7</authoring_tool>
    </contributor>
    <created>2020-08-31T16:02:53</created>
    <modified>2020-08-31T16:02:53</modified>
    <unit name="meter" meter="1"/>
    <up_axis>Z_UP</up_axis>
  </asset>
  <library_images/>
  <library_geometries>
    <geometry id="Cube_004-mesh" name="Cube.004">
      <mesh>
        <source id="Cube_004-mesh-positions">
          <float_array id="Cube_004-mesh-positions-array" count="36">-1.22211 -0.2 -0.8601566 -1.22211 -0.2 -0.06015658 -1.22211 0.2 -0.8601566 -1.22211 0.2 -0.06015658 0.7778905 -0.2 -0.8601566 0.7778905 -0.2 -0.06015658 0.7778905 0.2 -0.8601566 0.7778905 0.2 -0.06015658 -1.222109 -0.3 0.6222335 -1.222109 0.3 0.6222335 1.77789 0.3 0.6222335 1.77789 -0.3 0.6222335</float_array>
          <technique_common>
            <accessor source="#Cube_004-mesh-positions-array" count="12" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Cube_004-mesh-normals">
          <float_array id="Cube_004-mesh-normals-array" count="42">-1 0 0 0 1 0 1 0 0 0 -1 0 0 0 -1 0.5636593 0 -0.8260074 0 0 1 -1 0 1.74694e-7 0 -0.9894323 -0.1449952 0 0.9894324 -0.1449952 0.5636594 0 -0.8260074 -1 0 1.74694e-7 0 -0.9894324 -0.1449952 0 0.9894323 -0.1449952</float_array>
          <technique_common>
            <accessor source="#Cube_004-mesh-normals-array" count="14" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Cube_004-mesh-map-0">
          <float_array id="Cube_004-mesh-map-0-array" count="120">0.625 0 0.375 0.25 0.375 0 0.625 0.25 0.375 0.5 0.375 0.25 0.625 0.5 0.375 0.75 0.375 0.5 0.625 0.75 0.375 1 0.375 0.75 0.375 0.5 0.125 0.75 0.125 0.5 0.625 0.5 0.625 0.75 0.625 0.75 0.875 0.5 0.625 0.75 0.625 0.5 0.625 0 0.625 0.25 0.625 0.25 0.625 0.75 0.625 1 0.625 1 0.625 0.5 0.625 0.25 0.625 0.5 0.625 0 0.625 0.25 0.375 0.25 0.625 0.25 0.625 0.5 0.375 0.5 0.625 0.5 0.625 0.75 0.375 0.75 0.625 0.75 0.625 1 0.375 1 0.375 0.5 0.375 0.75 0.125 0.75 0.625 0.5 0.625 0.5 0.625 0.75 0.875 0.5 0.875 0.75 0.625 0.75 0.625 0 0.625 0 0.625 0.25 0.625 0.75 0.625 0.75 0.625 1 0.625 0.5 0.625 0.25 0.625 0.25</float_array>
          <technique_common>
            <accessor source="#Cube_004-mesh-map-0-array" count="60" stride="2">
              <param name="S" type="float"/>
              <param name="T" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <vertices id="Cube_004-mesh-vertices">
          <input semantic="POSITION" source="#Cube_004-mesh-positions"/>
        </vertices>
        <triangles count="20">
          <input semantic="VERTEX" source="#Cube_004-mesh-vertices" offset="0"/>
          <input semantic="NORMAL" source="#Cube_004-mesh-normals" offset="1"/>
          <input semantic="TEXCOORD" source="#Cube_004-mesh-map-0" offset="2" set="0"/>
          <p>1 0 0 2 0 1 0 0 2 3 1 3 6 1 4 2 1 5 7 2 6 4 2 7 6 2 8 5 3 9 0 3 10 4 3 11 6 4 12 0 4 13 2 4 14 7 5 15 11 5 16 5 5 17 9 6 18 11 6 19 10 6 20 1 7 21 9 7 22 3 7 23 5 8 24 8 8 25 1 8 26 7 9 27 9 9 28 10 9 29 1 0 30 3 0 31 2 0 32 3 1 33 7 1 34 6 1 35 7 2 36 5 2 37 4 2 38 5 3 39 1 3 40 0 3 41 6 4 42 4 4 43 0 4 44 7 10 45 10 10 46 11 10 47 9 6 48 8 6 49 11 6 50 1 11 51 8 11 52 9 11 53 5 12 54 11 12 55 8 12 56 7 13 57 3 13 58 9 13 59</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
  <library_controllers/>
</COLLADA>
        "##;

        let mut mesh = AnimatedMesh::default();
        let mut parser = ColladaReader::default();
        let mut cursor = Cursor::new(test);
        parser
            .read_into(&mut cursor, &mut mesh)
            .expect("It should not fail to pase that!");
    }
}
