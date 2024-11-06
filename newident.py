import zlib
import base64

#sidenote for compiling rust client "$env:RUSTFLAGS="-L C:\msys64\mingw64\lib""

def encode_ident_string(input_string):
    # Compress the input string using zlib
    compressed_data = zlib.compress(input_string.encode())
    
    # Encode the compressed data to base16 (hex)
    hex_encoded_data = base64.b16encode(compressed_data).decode()
    
    return hex_encoded_data

if __name__ == "__main__":
    # Prompt the user to enter the string to replace b_ident
    user_input = input("Enter the string to encode for b_ident: ")
    encoded_string = encode_ident_string(user_input)
    
    print(f"Encoded string to replace b_ident: {encoded_string}")

