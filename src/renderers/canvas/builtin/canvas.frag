#version 450

layout (location = 0) in vec4 fragColor;
layout (location = 1) in vec2 fragTexCoord;
layout (location = 2) flat in uint tex;
layout (location = 3) flat in uint instance;
layout (location = 4) flat in uint quad;
layout (location = 5) in vec2 xy;

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

#extension GL_EXT_nonuniform_qualifier: require
layout (set = 1, binding = 0) uniform sampler2D textures[];

struct Brush {
    vec4 fg;
    vec4 bg;
    vec4 radius;
    vec4 border;
// vec3 _unused;
};
layout (std140, set = 3, binding = 4) readonly buffer Style {
    Brush brushes[];
} style;

layout (location = 0) out vec4 outColor;

float rectangle(in vec2 p, in vec2 b, in vec4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x  = (p.y > 0.0) ? r.x  : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

void main() {
    Elem element = canvas.elements[instance];
    vec4 texColor = texture(textures[tex], fragTexCoord) * fragColor;

    switch (element.attrs[0]) {
        case 1:
        // RECTANGLE
        Brush brush = style.brushes[element.attrs[2]];
        vec4 fg = brush.fg * texColor;
        vec4 bg = brush.bg * texColor;
        float res = element.size.y;
        float border = brush.border[0];
        float borderFix = border / res;
        vec3 borderColor = border > 0.0 ? fg.rgb : bg.rgb;
        // substract border to fix rounded rectangle appearacne
        vec4 radius = brush.radius - vec4(border);
        // absolute smoothness reference is 100 px height
        float smoothness = (100 / res) * 0.001;
        vec2 offset = (xy - element.size / 2.0) / res;
        vec2 size = vec2(element.size.x / 2.0 / res, 0.5) - vec2(borderFix);
        float d = rectangle(offset, size, radius / res);
        vec3 rgb = (d > 0.0) ? vec3(1.0) : bg.rgb;
        float borderD = 1.0 - smoothstep(borderFix - smoothness, borderFix + smoothness, abs(d));
        rgb = mix(rgb, borderColor, borderD);
        outColor.rgb = rgb;
        outColor.a = (d > 0.0) ? borderD : 1.0;
        break;
        default :
        // IMAGE
        outColor = texColor;
        break;
    }

}