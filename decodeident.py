import base64
import zlib

def decompress_and_decode(hex_data):
    try:
        # Decode the hexadecimal data
        binary_data = base64.b16decode(hex_data, casefold=True)
        # Decompress using zlib and decode to utf-8
        return zlib.decompress(binary_data).decode('utf-8')
    except Exception as e:
        return f"An error occurred: {str(e)}"

b_ident = b"789CF3CBCC0DC849CC2B51703652084E2D2A4B2D02003B5C0650"
decoded_data = decompress_and_decode(b_ident)
print(decoded_data)
