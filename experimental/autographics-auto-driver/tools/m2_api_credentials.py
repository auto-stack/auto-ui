"""Independent pre-namespace API bytes for the five frozen M1 profiles.

Read actual API inputs; do not trust a generated snapshot's claimed api_hash or
confuse the final namespaced region with this compatibility credential.
"""
import pathlib


def compact(source):
    assert not any(c in source for c in ['"', "'", '`']), 'trusted API gained a literal'
    source=source.replace('\r\n','\n')
    # Match the pinned SDK comment transport, preserving newlines in comments.
    out=[];i=0
    while i<len(source):
        if source.startswith('//',i):
            end=source.find('\n',i+2)
            if end<0:break
            out.append('\n');i=end+1
        elif source.startswith('/*',i):
            end=source.find('*/',i+2)
            assert end>=0,'unterminated trusted API comment'
            out.append('\n'*source[i:end+2].count('\n'));i=end+2
        else:out.append(source[i]);i+=1
    text=';'.join(line.strip() for line in ''.join(out).split('\n') if line.strip())
    return text.replace('{;','{').replace(';}','}').replace(';;',';').replace('};fn ','}\nfn ')


def api_bytes(root,version):
    assert version in range(1,6),'unsupported API credential version'
    root=pathlib.Path(root)
    read=lambda name:(root/'api'/name).read_bytes().decode('utf-8')
    base=read('behavior_v1.at')
    if version==1:return base.encode('utf-8')
    spatial=read('behavior_spatial_v2.at')
    if version==2:return (base+'\n'+spatial).encode('utf-8')
    field='fields[node*12+axis]=value; _ag_dirty=1'
    movement='fields[node*12+2]=fields[node*12+2]+dz'
    visible='flags[node]=flag; _ag_dirty=1'
    assert all(base.count(marker)==1 for marker in [field,movement,visible]),'API setter boundary'
    base=base.replace(field,field+';if axis==0 {_ag_segment_dx=1};if axis==1 {_ag_segment_dy=1};if axis==2 {_ag_segment_dz=1}')
    base=base.replace(movement,movement+';_ag_segment_dx=1;_ag_segment_dy=1;_ag_segment_dz=1')
    base=base.replace(visible,visible+';_ag_segment_dv=1')
    segment=read('behavior_segment_v3.at')
    if version==3:return compact(base+'\n'+spatial+'\n'+segment).encode('utf-8')
    segment=('var _ag_query_kind=0\n'+segment).replace('_ag_cast_token=0','_ag_cast_token=0; _ag_query_kind=1')
    segment=segment.replace('if token<=0','if _ag_query_kind!=1 {_ag_fail(13,-1,0); return 0}; if token<=0')
    api=compact(base+'\n'+spatial+'\n'+segment+'\n'+read('behavior_sweep_v4.at'))
    if version==4:return api.encode('utf-8')
    marker=compact(field)
    assert marker in api,'motion setter boundary'
    api=api.replace(marker,compact('if _ag_motion_set_leaf(node,axis,value)==1 {return};'+field))
    return compact(api+'\n'+read('behavior_motion_scan_v5.at')+'\n'+read('behavior_motion_v5.at')).encode('utf-8')
