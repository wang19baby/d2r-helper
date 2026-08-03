import sys
data = open(r'D:\work_space\personal_workspace\d2r\开心邪帝.d2s', 'rb').read()
print('File size:', len(data))

for marker, name in [(b'gf', 'gf'), (b'if', 'if'), (b'JM', 'JM'), (b'jf', 'jf'), (b'kf', 'kf')]:
    pos = data.find(marker)
    if pos >= 0:
        print(f'{name} at 0x{pos:04X}')
    else:
        print(f'{name} NOT FOUND')

if_pos = data.find(b'if')
if if_pos >= 0:
    print(f'if segment (hex): {data[if_pos:if_pos+40].hex()}')

jm_pos = data.find(b'JM', if_pos + 2)
if jm_pos >= 0:
    print(f'JM at 0x{jm_pos:04X}')
    jm_count = int.from_bytes(data[jm_pos+2:jm_pos+4], 'little')
    print(f'JM count: {jm_count}')
    print(f'JM header (hex): {data[jm_pos:jm_pos+20].hex()}')
