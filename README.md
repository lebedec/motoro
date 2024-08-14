# motoro

A graphics engine, also known as a rendering engine, is a component of a game engine responsible for rendering 2D or 3D
graphics. It takes graphical assets (such as models, textures, lighting information) and outputs them to the screen in
real-time.

Functions:

+ Rendering 2D or 3D graphics
+ Managing graphical assets (models, textures, lighting)
+ Performing rendering tasks (e.g., triangle setup, vertex processing, pixel shading)
+ Interfacing with graphics drivers and hardware

TODO:

- add default 2D renderer with fonts (add shader from blob, test font loading)
- add bumaga to default 2D renderer
- design prefabs (remove get_texture/get_font)
- fix texture/samplers GLSL variable API
- do refactor, remove Vertex2D::no_input