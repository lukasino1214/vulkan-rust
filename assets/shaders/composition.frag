#version 450

layout (location = 0) out vec4 outColor;

layout (location = 0) in vec2 inUV;

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

layout (set = 1, binding = 0) uniform sampler2D position;
layout (set = 1, binding = 1) uniform sampler2D normal;
layout (set = 1, binding = 2) uniform sampler2D albedo;
layout (set = 1, binding = 3) uniform sampler2D metallic_roughness;

const float PI = 3.14159265359;

// ----------------------------------------------------------------------------
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}
// ----------------------------------------------------------------------------
void main()
{		

    vec4 albedo = texture(albedo, inUV);
    //vec4 albedo = vec4(1.0);

    float metallic = texture(metallic_roughness, inUV).r;
    float roughness = texture(metallic_roughness, inUV).g;

    vec3 position = texture(position, inUV).rgb;

    vec3 N = normalize(texture(normal, inUV).rgb);
    vec3 V = normalize(ubo.cameraPos - position);

    // calculate reflectance at normal incidence; if dia-electric (like plastic) use F0 
    // of 0.04 and if it's a metal, use the albedo color as F0 (metallic workflow)    

    vec3 fragColor = vec3(1.0);

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, fragColor, metallic);

    float bias = 1000.0f;

    // reflectance equation
    vec3 Lo = vec3(0.0);
    for (int i = 0; i < ubo.numLights; i++) {
        PointLight light = ubo.pointLights[i];
        vec3 L = normalize(light.position.xyz - position);
        vec3 H = normalize(V + L);
        float distance = length(light.position.xyz - position);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light.color.xyz * light.intensity * bias * attenuation;

        // Cook-Torrance BRDF
        float NDF = DistributionGGX(N, H, roughness);   
        float G   = GeometrySmith(N, V, L, roughness);      
        vec3 F    = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);
           
        vec3 numerator    = NDF * G * F; 
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
        vec3 specular = numerator / denominator;
        
        // kS is equal to Fresnel
        vec3 kS = F;
        // for energy conservation, the diffuse and specular light can't
        // be above 1.0 (unless the surface emits light); to preserve this
        // relationship the diffuse component (kD) should equal 1.0 - kS.
        vec3 kD = vec3(1.0) - kS;
        // multiply kD by the inverse metalness such that only non-metals 
        // have diffuse lighting, or a linear blend if partly metal (pure metals
        // have no diffuse light).
        kD *= 1.0 - metallic;	  

        // scale light by NdotL
        float NdotL = max(dot(N, L), 0.0);        

        // add to outgoing radiance Lo
        Lo += (kD * fragColor / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
    }
    // ambient lighting (note that the next IBL tutorial will replace 
    // this ambient lighting with environment lighting).
    vec3 ambient = vec3(0.03) * fragColor;

    vec3 color = ambient + Lo;

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    // gamma correct
    //color = pow(color, vec3(1.0/2.2)); 

    outColor = albedo * vec4(color, 1.0);
    //outColor = vec4(albedo.rgb, 1.0);
    //outColor = vec4(albedo.rgb, 1.0);
    //outColor = vec4(N.rgb, 1.0);
}