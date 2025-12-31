Much of the code is originally pulled from:
https://github.com/dfkeenan/SilkVulkanTutorial/tree/main/Source

```
sudo apt install vulkan-validationlayers
```

How to compile shaders from glsl source into spir-v binaries:
```
sudo apt install glslc
glslc Shaders/shader.vert -o Shaders/shader.vert.spv
glslc Shaders/shader.frag -o Shaders/shader.frag.spv
```
TODO makefile for shader compilation, or a compile step in the proj file xml