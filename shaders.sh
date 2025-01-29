#!./bash

glslc shaders/tetromino.vert -o shaders/bin/tetromino_vert.spv
glslc shaders/tetromino.frag -o shaders/bin/tetromino_frag.spv
glslc shaders/backdrop.frag -o shaders/bin/backdrop_frag.spv
glslc shaders/backdrop.vert -o shaders/bin/backdrop_vert.spv
glslc shaders/text.frag -o shaders/bin/text_frag.spv
glslc shaders/text.vert -o shaders/bin/text_vert.spv
glslc shaders/button.frag -o shaders/bin/button_frag.spv
glslc shaders/button.vert -o shaders/bin/button_vert.spv