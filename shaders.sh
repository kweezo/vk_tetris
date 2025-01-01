#!./bash

glslc shaders/tetromino.vert -o shaders/bin/tetromino_vert.spv
glslc shaders/tetromino.frag -o shaders/bin/tetromino_frag.spv
glslc shaders/backdrop.frag -o shaders/bin/backdrop_frag.spv
glslc shaders/backdrop.vert -o shaders/bin/backdrop_vert.spv