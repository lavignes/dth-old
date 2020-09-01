#!/usr/bin/env python3

from pathlib import Path
import os
import itertools
import subprocess

cwd = Path.cwd()
assert cwd == Path(__file__).resolve().parent, 'run this script from the directory it is in!'
subprocess.call('glslangValidator', stdout=subprocess.DEVNULL)


def compile_shader(path: Path):
    rel_path = path.relative_to(cwd)
    out_path = cwd.parent/'res'/'shaders'/rel_path
    out_path: Path = out_path.with_suffix(out_path.suffix + '.spv')
    # build the dir tree down to the output file, if necessary
    out_path.parent.mkdir(parents=True, exist_ok=True)
    modified_time = os.path.getmtime(path)
    # don't recompile if the modified timestamp hasn't changed
    if not out_path.exists() or modified_time != os.path.getmtime(out_path):
        subprocess.check_call(['glslangValidator', '-V100', '-o', out_path, path])
        # update modified timestamp on output to match input
        os.utime(out_path, (os.path.getatime(out_path), modified_time))


for shader in itertools.chain(cwd.glob('**/*.glsl'), cwd.glob('**/*.glsl')):
    compile_shader(shader)
