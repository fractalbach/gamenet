
import com.curiouscreature.kotlin.math.Double3
import info.laht.threekt.materials.ShaderMaterial
import info.laht.threekt.math.Vector3
import material.uValue

private const val terrainVert: String = """
attribute vec3 a_normal;
attribute vec3 a_tex_pos;
attribute float a_slope;
attribute float a_height;

varying float v_slope;
varying float v_height;
varying vec3 v_tex_pos;

void main() {
    v_slope = a_slope;
    v_height = a_height;
    v_tex_pos = a_tex_pos;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1);
}
"""

private const val terrainFrag: String = """
const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 0.9);
const vec3 DIRT_COLOR = vec3(0.74, 0.68, 0.46);
const vec3 GRASS_COLOR = vec3(0.38, 0.6, 0.15);

uniform vec3 u_fog_color;
uniform float u_fog_near;
uniform float u_fog_far;
uniform float u_grass_fog_far;

varying float v_slope;
varying float v_height;
varying vec3 v_tex_pos;

SIMPLEX!

void main() {
	vec3 color = GRASS_COLOR; // vec3(0.5, 0.5, 0.5);
    float simplex_value = simplex_noise(v_tex_pos * 3.0);
	simplex_value = mix(simplex_value, simplex_noise(v_tex_pos), 0.4);
    vec3 simplex_adj = vec3(simplex_value, simplex_value, simplex_value);
	color = mix(color, simplex_adj, 0.1);

	float depth = gl_FragCoord.z / gl_FragCoord.w;

	// then apply atmosphere fog
	float fog_factor = smoothstep(u_fog_near, u_fog_far, depth);
	color = mix(color, u_fog_color, fog_factor);
	gl_FragColor = vec4(color, 1.0);
}
"""

/**
 * Get terrain material instance.
 *
 * @param fog_color Double3(r, g, b) Where r, g, and b are between 0 and 1.
 * @param fog_near Fog start distance in meters.
 * @param fog_far Fog end distance in meters.
 */
fun getTerrainMat(
        fog_color: Double3, fog_near: Double, fog_far: Double
): ShaderMaterial {
    val uniforms: dynamic = object{}
    uniforms["u_fog_color"] = uValue(
            Vector3(fog_color.x, fog_color.y, fog_color.z)
    )
    uniforms["u_fog_near"] = uValue(fog_near)
    uniforms["u_fog_far"] = uValue(fog_far)
    uniforms["u_grass_fog_far"] = uValue(100)
    val options: dynamic = object{}
    options["uniforms"] = uniforms
    options["vertexShader"] = terrainVert
    options["fragmentShader"] = terrainFrag.replace("SIMPLEX!", simplex)
    return js("new THREE.ShaderMaterial(options)") as ShaderMaterial
}
