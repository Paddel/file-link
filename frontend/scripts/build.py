import os
import subprocess

current_directory = os.path.dirname(os.path.abspath(__file__))
parent_directory = os.path.abspath(os.path.join(current_directory, os.pardir))
os.chdir(parent_directory)

# subprocess.run(["wasm-pack", "build", "--target", "web", "--out-dir", "../backend/public"])
os.system('wasm-pack build --target web --out-dir ../backend/public')
os.system('rollup ./main.js --format iife --file ../backend/public/bundle.js')