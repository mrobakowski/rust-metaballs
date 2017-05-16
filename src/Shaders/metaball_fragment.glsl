#version 150

uniform vec3 light;
uniform mat4 view;

in vec4 normal;
in vec4 position;
out vec4 color;

const vec3 ambient_color = vec3(0.3, 0.1, 0.0);
const vec3 diffuse_color = vec3(0.6, 0.2, 0.0);
const vec3 specular_color = vec3(1.0, 0.5, 0.0);

void main() {
    vec3 v_light = vec3(view * vec4(light, 1));

    float diffuse = max(dot(normalize(normal.xyz), normalize(v_light - position.xyz)), 0.0);

    vec3 camera_dir = normalize(-position.xyz);
    vec3 half_direction = normalize(normalize(v_light - position.xyz) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(normal.xyz)), 0.0), 32.0);

    color = (vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0) 
        * 0.9 + normal * 0.1);// + inverse(view) * position * 0.3;
}
