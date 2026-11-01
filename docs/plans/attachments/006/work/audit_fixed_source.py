"""Local-only PLAN-006 source preflight; not the runtime admission oracle."""
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

root = Path(sys.argv[1])
sample = root / 'experimental/autographics-auto-driver/samples/textured_models'
manifest = Path('D:/autostack/plan-artifacts/auto-ui/plan-005/fixed-source-manifest.json')
raw_manifest = manifest.read_bytes()
assert hashlib.sha256(raw_manifest).hexdigest() == '74a88dbc289494f77b9be38180602e26bb7053f637d3ecbf26d2d0f43281b58c'
identity = json.loads(raw_manifest)
files = []
for item in identity['files']:
    name = {'README.md': 'SOURCE-README.md', 'LICENSE.md': 'SOURCE-LICENSE.md'}.get(item['path'], item['path'])
    data = (sample / name).read_bytes()
    assert len(data) == item['length']
    assert hashlib.sha256(data).hexdigest() == item['sha256'], name
    files.append({'path': name, 'bytes': len(data), 'sha256': item['sha256']})
b = (sample / 'models/character.glb').read_bytes()
assert struct.unpack_from('<III', b) == (0x46546C67, 2, len(b))
n, tag = struct.unpack_from('<II', b, 12)
assert tag == 0x4E4F534A
j = json.loads(b[20:20+n])
bin_length, bin_tag = struct.unpack_from('<II', b, 20+n)
assert bin_tag == 0x004E4942 and 28+n+bin_length == len(b)
blob = memoryview(b)[28+n:]
assert len(j['nodes']) <= 128 and not j.get('skins')
assert not any('skin' in node for node in j['nodes'])

def accessor(index, kind):
    a = j['accessors'][index]
    assert a['componentType'] == 5126 and a['type'] == kind and not a.get('sparse')
    v = j['bufferViews'][a['bufferView']]
    width = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[kind]
    stride = v.get('byteStride', width * 4)
    assert v.get('buffer', 0) == 0 and stride >= width * 4
    offset = a.get('byteOffset', 0)
    end = offset + (a['count']-1)*stride + width*4
    assert 0 <= offset <= end <= v['byteLength']
    start = v.get('byteOffset', 0)
    assert start + v['byteLength'] <= len(blob)
    rows = [struct.unpack_from('<'+'f'*width, blob, start+offset+i*stride) for i in range(a['count'])]
    assert all(math.isfinite(x) for row in rows for x in row)
    return rows

clips = []
total_keys = total_channels = sample_bytes = 0
for ordinal, clip in enumerate(j['animations']):
    seen = set()
    channels = []
    duration = 0
    for c in clip['channels']:
        node, path = c['target']['node'], c['target']['path']
        assert node < len(j['nodes']) and path in ('translation', 'rotation', 'scale')
        assert (node, path) not in seen
        seen.add((node, path))
        s = clip['samplers'][c['sampler']]
        mode = s.get('interpolation', 'LINEAR')
        assert mode in ('LINEAR', 'STEP')
        times = [row[0] for row in accessor(s['input'], 'SCALAR')]
        values = accessor(s['output'], 'VEC4' if path == 'rotation' else 'VEC3')
        assert times and times[0] >= 0 and all(y>x for x,y in zip(times, times[1:]))
        assert len(times) == len(values)
        if path == 'scale':
            assert all(x > 0 for row in values for x in row)
        if path == 'rotation':
            assert all(abs(sum(x*x for x in row)-1) < 1e-4 for row in values)
        duration = max(duration, times[-1])
        total_keys += len(times)
        sample_bytes += len(times) * (4 + len(values[0])*4)
        channels.append({'node': node, 'path': path, 'mode': mode, 'keys': len(times)})
    total_channels += len(channels)
    clips.append({'ordinal': ordinal, 'name': clip['name'], 'duration_seconds': duration, 'channels': channels})
assert [c['name'] for c in clips] == ['static', 'idle', 'walk', 'jump']
assert len(clips) <= 16 and total_channels <= 128 and total_keys <= 8192
print(json.dumps({'repository': identity['repository'], 'commit': identity['commit'], 'files': files,
    'nodes': j['nodes'], 'clips': clips, 'total_channels': total_channels,
    'total_channel_key_samples': total_keys, 'channel_time_and_output_bytes': sample_bytes,
    'scope': 'source preflight only; runtime/Auto/budget/GPU acceptance pending'}, indent=2))
