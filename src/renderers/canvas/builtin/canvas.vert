#version 450

struct Elem {
    vec2 position;
    vec2 image;
    vec2 src;
    vec2 uv;
    vec2 size;
//    uint kind;
//    uint texture;
//    uint brush;
    vec2 _unused;
    uvec4 attrs;

};
layout (std140, set = 0, binding = 4) readonly buffer Canvas {
    Elem elements[];
} canvas;

layout (set = 2, binding = 0) uniform UniformTranfrom {
    mat4 model;
    mat4 view;
    mat4 proj;
} transform;

struct Vertex {
    vec2 position;
    vec2 uv;
    uint q;
};

Vertex vertices[6] = Vertex[](
Vertex(vec2(0.0, 0.0), vec2(0.0, 0.0), 0),
Vertex(vec2(1.0, 0.0), vec2(1.0, 0.0), 1),
Vertex(vec2(1.0, 1.0), vec2(1.0, 1.0), 2),
Vertex(vec2(1.0, 1.0), vec2(1.0, 1.0), 2),
Vertex(vec2(0.0, 1.0), vec2(0.0, 1.0), 3),
Vertex(vec2(0.0, 0.0), vec2(0.0, 0.0), 0)
);

layout (location = 0) out vec4 fragColor;
layout (location = 1) out vec2 fragTexCoord;
layout (location = 2) flat out uint tex;
layout (location = 3) flat out uint instance;
layout (location = 4) flat out uint quad;
layout (location = 5) out vec2 xy;

void main() {
    mat4 camera = transform.proj * transform.view * transform.model;
    Elem element = canvas.elements[gl_InstanceIndex];
    vec2 position = vertices[gl_VertexIndex].position;
    vec2 uv = vertices[gl_VertexIndex].uv;
    gl_Position = camera * vec4((position * element.size * vec2(1) + element.position), 0.0, 1.0);
    fragColor = vec4(1.0);
    fragTexCoord = element.src + uv * element.uv;
    tex = element.attrs[1];
    instance = gl_InstanceIndex;
    xy = position * element.size;
    quad = vertices[gl_VertexIndex].q;
}