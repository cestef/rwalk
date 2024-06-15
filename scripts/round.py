# This script takes a GIF image, rounds the corners of each frame, and saves the result as a new GIF.
# Usage: python round.py <image> <radius> <output>
from PIL import Image, ImageDraw, ImageSequence
import sys

def round_corners(image_path, radius, output):
    image = Image.open(image_path)

    # Create a mask with rounded corners
    mask = Image.new("L", image.size, 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, image.width, image.height), radius, fill=255)

    frames = []
    for frame in ImageSequence.Iterator(image):
        result = Image.new("RGBA", frame.size)
        result.paste(frame, mask=mask)
        frames.append(result)

    frames[0].save(
        output,
        save_all=True,
        append_images=frames[1:],
        loop=image.info["loop"],
        duration=image.info["duration"],
    )


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python round.py <image> <radius> <output>")
        sys.exit(1)

    image_path = sys.argv[1]
    radius = int(sys.argv[2])
    output = sys.argv[3]
    round_corners(image_path, radius, output)
