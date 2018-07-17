#version 420

// we get grid points as input and output the triangles that result from running marching cubes on the computed field
layout(points) in; 
layout (triangle_strip, max_vertices = 15) out; // maximally 5 triangles

uniform mat4 perspective, view;

uniform isamplerBuffer triTableTex; // array containing the marching cubes data - ./marching_cubes_data/mod.rs
uniform samplerBuffer metaballs; // array containing metaball position (xyz) and metaball intensity (w)
uniform int metaballsLength; // current length of the array above
uniform float isolevel; // the field value the surface should be drawn at

uniform float cubeSideLength; // cell size

out vec4 normal;
out vec4 position;

vec3 cubeVertex(int i) {
    switch (i) {
        case 0: return vec3(0, 0, 0);
        case 1: return vec3(cubeSideLength, 0, 0);
        case 2: return vec3(cubeSideLength, cubeSideLength, 0);
        case 3: return vec3(0, cubeSideLength, 0);
        case 4: return vec3(0, 0, cubeSideLength);
        case 5: return vec3(cubeSideLength, 0, cubeSideLength);
        case 6: return vec3(cubeSideLength, cubeSideLength, cubeSideLength);
        case 7: return vec3(0, cubeSideLength, cubeSideLength);
    }
    return vec3(0, 0, 0);
}

vec3 cubePos(int i, vec3 cur_origin) {
    return cur_origin + cubeVertex(i);
}

float distSq(vec3 a, vec3 b) {
    vec3 amb = a - b;
    vec3 amb2 = amb * amb;
    return amb2.x + amb2.y + amb2.z;
}

float fieldData(vec3 where) {
    if (abs(where.x) > 1.9 || abs(where.y) > 1.9 || abs(where.z) > 1.9) return 0.0;
    float res = 0.0;
    for (int j = 0; j < metaballsLength; ++j) {
        vec4 metaball = texelFetch(metaballs, j);
        float rSq = distSq(where, metaball.xyz);
        res += (1 - rSq) * (1 - rSq) * metaball.w * float(rSq < 1.0);
    }
    return res;
}

float cubeVal(int i, vec3 cur_origin) {
    vec3 cubePosTmp = cubePos(i, cur_origin);
    return fieldData(cubePosTmp);
}

int triTableValue(int i, int j) {
    int off = i*16 + j;
    return texelFetch(triTableTex, off/4)[off%4];
}

vec3 vertexInterp(float isolevel, vec3 v0, float l0, vec3 v1, float l1) {
    return mix(v0, v1, (isolevel-l0)/(l1-l0));
}

void main()
{
    vec3 cur_origin = gl_in[0].gl_Position.xyz;
    int cubeindex = 0;

    float cubeVal0 = cubeVal(0, cur_origin);
    float cubeVal1 = cubeVal(1, cur_origin);
    float cubeVal2 = cubeVal(2, cur_origin);
    float cubeVal3 = cubeVal(3, cur_origin);
    float cubeVal4 = cubeVal(4, cur_origin);
    float cubeVal5 = cubeVal(5, cur_origin);
    float cubeVal6 = cubeVal(6, cur_origin);
    float cubeVal7 = cubeVal(7, cur_origin);

    cubeindex = int(cubeVal0 < isolevel);
    cubeindex += int(cubeVal1 < isolevel)*2;
    cubeindex += int(cubeVal2 < isolevel)*4;
    cubeindex += int(cubeVal3 < isolevel)*8;
    cubeindex += int(cubeVal4 < isolevel)*16;
    cubeindex += int(cubeVal5 < isolevel)*32;
    cubeindex += int(cubeVal6 < isolevel)*64;
    cubeindex += int(cubeVal7 < isolevel)*128;

    if (cubeindex == 0 || cubeindex == 255)
        return;

    vec3 vertlist[12];

    vertlist[0]  = vertexInterp(isolevel, cubePos(0, cur_origin), cubeVal0, cubePos(1, cur_origin), cubeVal1);
    vertlist[1]  = vertexInterp(isolevel, cubePos(1, cur_origin), cubeVal1, cubePos(2, cur_origin), cubeVal2);
    vertlist[2]  = vertexInterp(isolevel, cubePos(2, cur_origin), cubeVal2, cubePos(3, cur_origin), cubeVal3);
    vertlist[3]  = vertexInterp(isolevel, cubePos(3, cur_origin), cubeVal3, cubePos(0, cur_origin), cubeVal0);
    vertlist[4]  = vertexInterp(isolevel, cubePos(4, cur_origin), cubeVal4, cubePos(5, cur_origin), cubeVal5);
    vertlist[5]  = vertexInterp(isolevel, cubePos(5, cur_origin), cubeVal5, cubePos(6, cur_origin), cubeVal6);
    vertlist[6]  = vertexInterp(isolevel, cubePos(6, cur_origin), cubeVal6, cubePos(7, cur_origin), cubeVal7);
    vertlist[7]  = vertexInterp(isolevel, cubePos(7, cur_origin), cubeVal7, cubePos(4, cur_origin), cubeVal4);
    vertlist[8]  = vertexInterp(isolevel, cubePos(0, cur_origin), cubeVal0, cubePos(4, cur_origin), cubeVal4);
    vertlist[9]  = vertexInterp(isolevel, cubePos(1, cur_origin), cubeVal1, cubePos(5, cur_origin), cubeVal5);
    vertlist[10] = vertexInterp(isolevel, cubePos(2, cur_origin), cubeVal2, cubePos(6, cur_origin), cubeVal6);
    vertlist[11] = vertexInterp(isolevel, cubePos(3, cur_origin), cubeVal3, cubePos(7, cur_origin), cubeVal7);

    // probably should be precomputed and passed down to the shader
    mat3 normal_mat = transpose(inverse(mat3(view)));


    for (int i = 0; i < 15; i += 3) {
        int ttv0 = triTableValue(cubeindex, i);
        if (ttv0 == -1) break;
        vec4 pos0 = vec4(vertlist[ttv0], 1);

        int ttv1 = triTableValue(cubeindex, i + 1);
        if (ttv1 == -1) break;
        vec4 pos1 = vec4(vertlist[ttv1], 1);

        int ttv2 = triTableValue(cubeindex, i + 2);
        if (ttv2 == -1) break;
        vec4 pos2 = vec4(vertlist[ttv2], 1);

        normal = vec4(normalize(-vec3(
            fieldData(pos0.xyz + vec3(cubeSideLength/2, 0, 0)) - fieldData(pos0.xyz + vec3(-cubeSideLength/2, 0, 0)),
            fieldData(pos0.xyz + vec3(0, cubeSideLength/2, 0)) - fieldData(pos0.xyz + vec3(0, -cubeSideLength/2, 0)),
            fieldData(pos0.xyz + vec3(0, 0, cubeSideLength/2)) - fieldData(pos0.xyz + vec3(0, 0, -cubeSideLength/2)))), 1);
        normal = vec4(normal_mat * normal.xyz, 1);
        gl_Position = perspective * view * pos0;
        position = view * pos0;
        EmitVertex();

        normal = vec4(normalize(-vec3(
            fieldData(pos1.xyz + vec3(cubeSideLength/2, 0, 0)) - fieldData(pos1.xyz + vec3(-cubeSideLength/2, 0, 0)),
            fieldData(pos1.xyz + vec3(0, cubeSideLength/2, 0)) - fieldData(pos1.xyz + vec3(0, -cubeSideLength/2, 0)),
            fieldData(pos1.xyz + vec3(0, 0, cubeSideLength/2)) - fieldData(pos1.xyz + vec3(0, 0, -cubeSideLength/2)))), 1);
        normal = vec4(normal_mat * normal.xyz, 1);
        gl_Position = perspective * view * pos1;
        position = view * pos1;
        EmitVertex();

        normal = vec4(normalize(-vec3(
            fieldData(pos2.xyz + vec3(cubeSideLength/2, 0, 0)) - fieldData(pos2.xyz + vec3(-cubeSideLength/2, 0, 0)),
            fieldData(pos2.xyz + vec3(0, cubeSideLength/2, 0)) - fieldData(pos2.xyz + vec3(0, -cubeSideLength/2, 0)),
            fieldData(pos2.xyz + vec3(0, 0, cubeSideLength/2)) - fieldData(pos2.xyz + vec3(0, 0, -cubeSideLength/2)))), 1);
        normal = vec4(normal_mat * normal.xyz, 1);
        gl_Position = perspective * view * pos2;
        position = view * pos2;
        EmitVertex();

        EndPrimitive();
    }
}
