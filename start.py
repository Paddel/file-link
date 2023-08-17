import os
import subprocess

import subprocess
import os

current_directory = os.path.dirname(os.path.abspath(__file__))

print('Building and bundling frontend...')
os.system('python frontend/scripts/build.py')
print('Suilding and starting backend...')
os.chdir(f'{current_directory}/backend')
os.system('cargo run')