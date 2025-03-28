#!/usr/bin/env nix-shell
#!nix-shell -i python -p "python3.withPackages (pkgs: with pkgs.pythonPackages; [ pillow ])"

###########
# Imports #
###########

# Default
import io
import os

# Maths
import math
import itertools

# Web
import urllib.request
import urllib.parse
import json

# Images
from PIL import Image, ImageDraw

# Server
from http.server import BaseHTTPRequestHandler, HTTPServer

###########
# Globals #
###########

# Path
CURRENT_DIR = os.path.dirname(__file__)

# Server
PORT = 8080

# Sizing
TILE_SIZE = 256  # Pixels
# Icon
ICON_POINT_X = 0.5
ICON_POINT_Y = 1

# Offsets needed for full rendering of a tile
OFFSETS_X = [-1, 0, 1]
OFFSETS_Y = [-1, 0, 1]

# Page
BASE_PROTOCOL = "https"
BASE_URL = "embed.waze.com"
BASE_PATH = "live-map/api/georss"
BASE_VARIABLES = {
    "env": "row"
}
BASE_TYPES = [ "alerts" ]

# Assets
ASSETS_DEFAULT = "DEFAULT"
ASSETS_EXTENSION = "png"
ASSETS_PATH = "assets"


#############
# Functions #
#############


def load_image(file_name: str):
    file_path = os.path.join(CURRENT_DIR, ASSETS_PATH, f"{file_name}.{ASSETS_EXTENSION}")
    return Image.open(file_path)


def correlate(maintype: str, subtype: str) -> str:
    if maintype in ASSETS_CORRELATION:
        if subtype in ASSETS_CORRELATION[maintype]:
            return ASSETS_CORRELATION[maintype][subtype]
        else:
            return ASSETS_CORRELATION[maintype][ASSETS_DEFAULT]
    else:
        return None


# Calculate the proper coordinates
def calculate_coordinate(x, y, z):

    # Size
    n = 2 ** z

    # Calculation
    lon = ((x / n) * 360.0) - 180.0
    lat = math.atan(math.sinh(math.pi * (1 - 2 * (y / n)))) * (180 / math.pi)

    # Return
    return lat, lon

def find_location(lon, lat, top, left, bottom, right, h_size, v_size):

    # Calculate ratio
    horizontal = h_size / (left - right)
    vertical = v_size / (top - bottom)

    # Item offset
    h_item = (left - lon) * horizontal
    v_item = (top - lat) * vertical

    return math.floor(h_item), math.floor(v_item)

def translate_edge(width, height, x, y, ratio_x, ratio_y):

    # Offsets
    offset_x = width * ratio_x
    offset_y = height * ratio_y

    # Translation
    translated_x = x - offset_x
    translated_y = y - offset_y

    # Check if invalid
    if (translated_x < 0) or (translated_y < 0):
        return None

    return int(translated_x), int(translated_y)


##########
# Assets #
##########


# Assets Correlation
ASSETS_CORRELATION = {
    "JAM": {
        ASSETS_DEFAULT: load_image("traffic-low"),
        "JAM_HEAVY_TRAFFIC": load_image("traffic-low"),
        "JAM_STAND_STILL_TRAFFIC": load_image("traffic-high")
    },
    "HAZARD": {
        ASSETS_DEFAULT: load_image("hazard"),
        "HAZARD_ON_ROAD_POT_HOLE": load_image("pothole"),
        "HAZARD_ON_ROAD_CONSTRUCTION": load_image("construction"),
        "HAZARD_ON_ROAD_ICE": load_image("ice"),
        "HAZARD_ON_ROAD_TRAFFIC_LIGHT_FAULT": load_image("light"),
        "HAZARD_ON_ROAD_OBJECT": load_image("object"),
        "HAZARD_ON_SHOULDER_CAR_STOPPED": load_image("vehicle-stopped"),
        "HAZARD_WEATHER_FOG": load_image("fog")
    },
    "ROAD_CLOSED": {
        ASSETS_DEFAULT: load_image("closure")
    },
    "ACCIDENT": {
        ASSETS_DEFAULT: load_image("accident")
    },
    "POLICE": {
        ASSETS_DEFAULT: load_image("police")
    },
    ASSETS_DEFAULT: {
        ASSETS_DEFAULT: load_image("simple")
    }
}
ASSETS_WIDTH, ASSETS_HEIGHT = \
    ASSETS_CORRELATION[ASSETS_DEFAULT][ASSETS_DEFAULT].size


########
# Main #
########


def waze_tile(given_x, given_y, given_z):

    # Store all alerts
    alerts = []

    # Get coordinates
    top, left = calculate_coordinate(
        given_x + min(OFFSETS_X),
        given_y + min(OFFSETS_Y),
        given_z
    )
    bottom, right = calculate_coordinate(
        given_x + max(OFFSETS_X) + 1,
        given_y + max(OFFSETS_Y) + 1,
        given_z
    )

    # Request
    arguments = {
        "top": top,
        "left": left,
        "bottom": bottom,
        "right": right,
        "types": ",".join(BASE_TYPES)
    } | BASE_VARIABLES

    encoded = urllib.parse.urlencode(query=arguments, doseq=True)
    url_now = urllib.parse.urlunsplit((BASE_PROTOCOL, BASE_URL, BASE_PATH, encoded, ""))

    # Store data temporarily
    waze_data = {}

    # Extract data
    with urllib.request.urlopen(url_now) as url:
        waze_data = json.load(url)

    # Extract Alerts
    if "alerts" in waze_data:
        for each in waze_data["alerts"]:
            icon = correlate(each["type"], each["subtype"])
            if icon:
                alerts.append({ "icon": icon, "coordinates": each["location"] })

    # Generate tile conglomerate transparent
    image_width = TILE_SIZE * len(OFFSETS_X)
    image_height = TILE_SIZE * len(OFFSETS_Y)
    image = Image.new(
        "RGBA",
        (image_width, image_height),
        (0, 0, 0, 0)
    )

    # Sort because of overlap
    sorted_alerts = sorted(
        alerts,
        key = lambda each: (
            each["coordinates"]["y"],
            each["coordinates"]["x"]
        )
    )

    # Iterate the alerts and add them to the image
    for alert in sorted_alerts:

        # Translate image
        location_h, location_v = find_location(
            alert["coordinates"]["x"], alert["coordinates"]["y"],
            top, left, bottom, right,
            image_width, image_height
        )

        # Fix edge location
        edges = translate_edge(
            ASSETS_WIDTH, ASSETS_HEIGHT,
            location_h, location_v,
            ICON_POINT_X, ICON_POINT_Y
        )

        if edges:
            image.alpha_composite(alert["icon"], edges)

    # Crop image to the right size
    edge_h = OFFSETS_X.index(0) * TILE_SIZE
    edge_v = OFFSETS_Y.index(0) * TILE_SIZE
    cropped = image.crop((
        # Left, Upper, Right, Lower
        edge_h, edge_v,
        edge_h + TILE_SIZE, edge_v + TILE_SIZE
    ))

    # Return it
    return cropped


# Web Handler
class WebHandler(BaseHTTPRequestHandler):

    def do_GET(self):

        # Verbose
        print("GET")

        # Query filtering
        query_components = urllib.parse.parse_qs(
            urllib.parse.urlparse(self.path).query
        )

        # Check parameters
        if not all(each in query_components for each in ["x", "y", "z"]):
            self.send_response(400)
            return

        # Assign to variables
        sent_x = query_components["x"][0]
        sent_y = query_components["y"][0]
        sent_z = query_components["z"][0]

        # Check if numeric
        for each in [sent_x, sent_y, sent_z]:
            if not each.isdecimal():
                self.send_response(400)
                return

        # Convert them
        try:
            sent_x = int(sent_x)
            sent_y = int(sent_y)
            sent_z = int(sent_z)
        except ValueError:
            self.send_response(400)
            return

        # Check range
        max_num = 0xFFFFFFFF
        for each in [sent_x, sent_y, sent_z]:
            if (each < 0) or (each > max_num):
                self.send_response(400)
                return

        # Respond
        self.send_response(200)
        self.send_header("Content-type", "image/png")
        self.end_headers()

        # Create Image
        image = waze_tile(sent_x, sent_y, sent_z)

        image.save("test_tmp.png", ASSETS_EXTENSION.upper())

        virtual = io.BytesIO()
        image.save(virtual, ASSETS_EXTENSION.upper())

        self.wfile.write(virtual.getbuffer())

        return


# Main
if __name__ == "__main__":

    try:
        server = HTTPServer(("", PORT), WebHandler)
        print(f'HTTP Running on {PORT}')
        server.serve_forever()

    # Interval
    except KeyboardInterrupt:
        print('^C Received, Shutting Down Server')
        server.socket.close()