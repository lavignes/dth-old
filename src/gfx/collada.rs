use crate::{
    collections::XorHashMap,
    gfx::{StaticMaterialMesh, StaticMaterialVertex},
    math::{Vector2, Vector3, Vector4},
    util::{self},
};
use std::{
    io::{self, ErrorKind, Read},
    iter,
};
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
    kind: Option<SourceKind>,
}

#[derive(Debug)]
enum TriangleInputKind {
    Vertex,
    Normal,
    TexCoord,
    Color,
}

#[derive(Debug)]
struct TriangleInput {
    kind: TriangleInputKind,
    offset: usize,
}

/// A quick and dirty collada parser
///
/// Long term this will be used to make a converter to a custom format.
///
/// # Limitations
/// - Single geometry with single mesh
#[derive(Debug, Default)]
pub struct ColladaReader {
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

    // Buffer to store normals
    normals: Vec<Vector3>,

    // Buffer to store tex coords
    tex_coords: Vec<Vector2>,

    // Buffer to store colors
    colors: Vec<Vector4>,
}

impl ColladaReader {
    pub fn read_into<R: Read>(
        &mut self,
        reader: &mut R,
        mesh: &mut StaticMaterialMesh,
    ) -> io::Result<()> {
        mesh.clear();

        self.states.clear();
        self.sources.clear();
        self.vertices_mapping.clear();
        self.triangle_inputs.clear();
        self.latest_id.clear();
        self.primitive_indices.clear();
        self.positions.clear();
        self.normals.clear();
        self.tex_coords.clear();
        self.colors.clear();
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
        // TODO: Can probably replace all EndDocument events with a single pop
        loop {
            let state =
                util::io_err_option(self.top(), ErrorKind::Other, || "Parser in invalid state")?;
            let event = util::io_err_result(xml_reader.next(), ErrorKind::Other)?;

            match event {
                XmlEvent::EndElement { .. } => {
                    self.pop();
                    continue;
                }
                XmlEvent::EndDocument { .. } => break,
                _ => (),
            }

            match state {
                State::Init => match event {
                    XmlEvent::StartDocument { .. } => {
                        self.push(State::ColladaTag);
                    }
                    _ => unimplemented!("{:?}", event),
                },

                // If we are in a tag that we dont implement, we obviously
                // dont support its sub-tags.
                State::UnimplementedTagLevel => match event {
                    XmlEvent::StartElement { .. } => {
                        self.push(State::UnimplementedTagLevel);
                    }
                    XmlEvent::Characters(_) => {}
                    _ => unimplemented!("{:?}", event),
                },

                State::ColladaTag => match event {
                    XmlEvent::StartElement { name, .. } => {
                        if name.local_name != "COLLADA" {
                            return util::io_err(
                                ErrorKind::InvalidData,
                                "First tag is supposed to be COLLADA",
                            );
                        }
                        self.push(State::Libraries);
                    }
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
                    _ => unimplemented!("{:?}", event),
                },

                State::MeshChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "source" => {
                            let id = util::io_err_option(
                                Self::find_attribute(&attributes, "id"),
                                ErrorKind::InvalidData,
                                || "Mesh sources must have ids",
                            )?;
                            self.set_latest_id(&id.value);
                            self.sources.insert(id.value.clone(), Source::default());
                            self.push(State::SourceChild);
                        }
                        "vertices" => {
                            let id = util::io_err_option(
                                Self::find_attribute(&attributes, "id"),
                                ErrorKind::InvalidData,
                                || "Mesh vertices must have ids",
                            )?;
                            self.set_latest_id(&id.value);
                            self.push(State::VerticesChild);
                        }
                        "triangles" => self.push(State::TrianglesChild),
                        _ => {
                            self.push(State::UnimplementedTagLevel);
                        }
                    },
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
                    _ => unimplemented!("{:?}", event),
                },

                State::SourceFloatArrayText => match event {
                    XmlEvent::Characters(text) => {
                        let source = util::io_err_option(
                            self.sources.get_mut(&self.latest_id),
                            ErrorKind::Other,
                            || "Parser in invalid state",
                        )?;
                        let mut floats = Vec::new();
                        for float in text.split_whitespace() {
                            floats.push(util::parse(float)?);
                        }
                        source.kind = Some(SourceKind::FloatArray(floats));
                    }
                    _ => unimplemented!("{:?}", event),
                },

                State::VerticesChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "input" => {
                            let source = util::io_err_option(
                                Self::find_attribute(&attributes, "source"),
                                ErrorKind::InvalidData,
                                || "Vertex inputs must have a source",
                            )?;
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
                    _ => unimplemented!("{:?}", event),
                },

                State::TrianglesChild => match event {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_str() {
                        "input" => {
                            // The semantic is like the type (vertex, normal, tex coord, etc).
                            let semantic = util::io_err_option(
                                Self::find_attribute(&attributes, "semantic"),
                                ErrorKind::InvalidData,
                                || "Triangles inputs must have a semantic",
                            )?;
                            let source = util::io_err_option(
                                Self::find_attribute(&attributes, "source"),
                                ErrorKind::InvalidData,
                                || "Triangles inputs must have a source",
                            )?;
                            let offset = util::parse(
                                &util::io_err_option(
                                    Self::find_attribute(&attributes, "offset"),
                                    ErrorKind::InvalidData,
                                    || "Triangles inputs must have a offset",
                                )?
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
                                    "COLOR" => TriangleInput {
                                        offset,
                                        kind: TriangleInputKind::Color,
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
                            let source = util::io_err_option(
                                self.sources.get(id),
                                ErrorKind::Other,
                                || "Input source no longer exists",
                            )?;
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
                                            let offset = index * 3;
                                            self.positions.push(
                                                (
                                                    positions[offset],
                                                    positions[offset + 1],
                                                    positions[offset + 2],
                                                )
                                                    .into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                    TriangleInputKind::Normal => match &source.kind {
                                        Some(SourceKind::FloatArray(normals)) => {
                                            let offset = index * 3;
                                            self.normals.push(
                                                (
                                                    normals[offset],
                                                    normals[offset + 1],
                                                    normals[offset + 2],
                                                )
                                                    .into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                    TriangleInputKind::TexCoord => match &source.kind {
                                        Some(SourceKind::FloatArray(tex_coords)) => {
                                            let offset = index * 2;
                                            self.tex_coords.push(
                                                (tex_coords[offset], tex_coords[offset + 1]).into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                    TriangleInputKind::Color => match &source.kind {
                                        Some(SourceKind::FloatArray(colors)) => {
                                            let offset = index * 4;
                                            self.colors.push(
                                                (
                                                    colors[offset],
                                                    colors[offset + 1],
                                                    colors[offset + 2],
                                                    colors[offset + 3],
                                                )
                                                    .into(),
                                            );
                                        }
                                        k => unimplemented!("{:?}", k),
                                    },
                                }
                            }
                        }
                    }
                    _ => unimplemented!("{:?}", event),
                },
            }
        }

        // TODO: This is kind of gross.
        //  This gives is a default white color
        let white = Vector4::splat(1.0);
        let mut white_iter = iter::repeat(&white);
        let mut colors_iter = self.colors.iter();
        let colors: &mut dyn Iterator<Item = &Vector4> = if self.colors.is_empty() {
            &mut white_iter // 'as &mut dyn Iterator<Item = &Vector4>' is also legal here!
        } else {
            &mut colors_iter
        };

        // TODO: Compress like indices
        for (i, (((&position, &normal), &tex_coord), &color)) in self
            .positions
            .iter()
            .zip(self.normals.iter())
            .zip(self.tex_coords.iter())
            .zip(colors)
            .enumerate()
        {
            mesh.add_vertex(StaticMaterialVertex::new(
                position, normal, tex_coord, color,
            ));
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

    /// Helper to remove the hashtag on reftag strings "#example" -> "example"
    #[inline]
    fn trim_ref(value: &str) -> String {
        value[1..].to_owned()
    }
}

#[cfg(test)]
mod test {
    use crate::gfx::{ColladaReader, StaticMaterialMesh};
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
    <created>2020-09-20T16:23:45</created>
    <modified>2020-09-20T16:23:45</modified>
    <unit name="meter" meter="1"/>
    <up_axis>Z_UP</up_axis>
  </asset>
  <library_images/>
  <library_geometries>
    <geometry id="Plane-mesh" name="Plane">
      <mesh>
        <source id="Plane-mesh-positions">
          <float_array id="Plane-mesh-positions-array" count="12">-1 -1 0 1 -1 0 -1 1 0 1 1 0</float_array>
          <technique_common>
            <accessor source="#Plane-mesh-positions-array" count="4" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Plane-mesh-normals">
          <float_array id="Plane-mesh-normals-array" count="3">0 0 1</float_array>
          <technique_common>
            <accessor source="#Plane-mesh-normals-array" count="1" stride="3">
              <param name="X" type="float"/>
              <param name="Y" type="float"/>
              <param name="Z" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Plane-mesh-map-0">
          <float_array id="Plane-mesh-map-0-array" count="12">1 0 0 1 0 0 1 0 1 1 0 1</float_array>
          <technique_common>
            <accessor source="#Plane-mesh-map-0-array" count="6" stride="2">
              <param name="S" type="float"/>
              <param name="T" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <source id="Plane-mesh-colors-Col" name="Col">
          <float_array id="Plane-mesh-colors-Col-array" count="24">0.4549019 0.7764706 0.2980392 1 0.4549019 0.7764706 0.2980392 1 0.2941176 0.1058824 0.7647059 1 0.4549019 0.7764706 0.2980392 1 1 0.2901961 0.2901961 1 0.4549019 0.7764706 0.2980392 1</float_array>
          <technique_common>
            <accessor source="#Plane-mesh-colors-Col-array" count="6" stride="4">
              <param name="R" type="float"/>
              <param name="G" type="float"/>
              <param name="B" type="float"/>
              <param name="A" type="float"/>
            </accessor>
          </technique_common>
        </source>
        <vertices id="Plane-mesh-vertices">
          <input semantic="POSITION" source="#Plane-mesh-positions"/>
        </vertices>
        <triangles count="2">
          <input semantic="VERTEX" source="#Plane-mesh-vertices" offset="0"/>
          <input semantic="NORMAL" source="#Plane-mesh-normals" offset="1"/>
          <input semantic="TEXCOORD" source="#Plane-mesh-map-0" offset="2" set="0"/>
          <input semantic="COLOR" source="#Plane-mesh-colors-Col" offset="3" set="0"/>
          <p>1 0 0 0 2 0 1 1 0 0 2 2 1 0 3 3 3 0 4 4 2 0 5 5</p>
        </triangles>
      </mesh>
    </geometry>
  </library_geometries>
  <library_visual_scenes>
    <visual_scene id="Scene" name="Scene">
      <node id="Plane" name="Plane" type="NODE">
        <matrix sid="transform">21.93946 0 0 0 0 21.93946 0 0 0 0 21.93946 0 0 0 0 1</matrix>
        <instance_geometry url="#Plane-mesh" name="Plane"/>
      </node>
    </visual_scene>
  </library_visual_scenes>
  <scene>
    <instance_visual_scene url="#Scene"/>
  </scene>
</COLLADA>
        "##;

        let mut mesh = StaticMaterialMesh::default();
        let mut parser = ColladaReader::default();
        let mut cursor = Cursor::new(test);
        parser
            .read_into(&mut cursor, &mut mesh)
            .expect("It should not fail to parse that!");
    }
}
