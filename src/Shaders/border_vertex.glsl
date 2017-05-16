#version 140

in vec3 position;
in vec2 tex_coord;

uniform mat4 view;
uniform mat4 persp;

out vec2 texCoord;

void main(void)
{
    texCoord = tex_coord;
	gl_Position = persp * view * vec4(position, 1);
}
