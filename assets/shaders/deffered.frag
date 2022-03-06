#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragPosWorld;
layout(location = 2) in vec3 fragNormalWorld;
layout(location = 3) in vec4 fragTangentWorld;
layout(location = 4) in vec2 fragUV;

layout (location = 0) out vec4 outPosition;
layout (location = 1) out vec4 outNormal;
layout (location = 2) out vec4 outAlbedo;
layout (location = 3) out vec4 outMetallicRoughness;

struct PointLight {
  vec4 position; // ignore w
  vec3 color; // w is intensity
  float intensity;
};

layout(set = 0, binding = 0) uniform GlobalUbo {
  mat4 projection;
  mat4 view;
  vec3 cameraPos;
  vec4 ambientLightColor; // w is intensity
  PointLight pointLights[10];
  int numLights;
} ubo;

layout(set = 1, binding = 0) uniform sampler2D albedo;
layout(set = 1, binding = 1) uniform sampler2D metallic_roughness;
layout(set = 1, binding = 2) uniform sampler2D normal;
layout(set = 1, binding = 3) uniform sampler2D occlusion;
layout(set = 1, binding = 4) uniform sampler2D emissive;
layout(set = 1, binding = 5) uniform PbrUbo {
  vec3 albedo;
  float metallic;
  float roughness;
  vec3 emissive; // w is intensity
} pbr;

layout(push_constant) uniform Push {
  mat4 modelMatrix;
  mat4 normalMatrix;
} push;

void main() {		
    vec3 N = normalize(fragNormalWorld);
    vec3 T = normalize(fragTangentWorld.xyz);
    vec3 B = cross(fragNormalWorld, fragTangentWorld.xyz) * fragTangentWorld.w;
    mat3 TBN = mat3(T, B, N);
    N = TBN * normalize(texture(normal, fragUV).xyz * 2.0 - vec3(1.0));
    //N = normalize(fragNormalWorld);

  vec4 color = texture(albedo, fragUV);
  if(color.w < 0.0001) {
    discard;
  }

  outPosition = vec4(fragPosWorld, 1.0);
  outAlbedo = color;
  outMetallicRoughness = texture(metallic_roughness, fragUV);
  outNormal = vec4(N, 1.0);
}