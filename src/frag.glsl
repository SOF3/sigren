precision mediump float;

in vec2 uvs;

uniform mat4 p;

uniform float unif_k[NUM_SIGNALS];
uniform vec3 unif_s[NUM_SIGNALS];
uniform vec3 unif_b[NUM_SIGNALS];

uniform float unif_lower;
uniform float unif_upper;

layout (location = 0) out lowp vec4 color;

float sq(float f) {
	return f * f;
}

float f_comp(vec3 c, vec3 s, int i, int j) {
	return p[2][i] * p[2][j] * s[i] * s[j] * sq(c[i] / p[2][i] - c[j] / p[2][j]);
}

float integral(int i) {
	float k = unif_k[i];
	vec3 s = unif_s[i];
	vec3 b = unif_b[i];

	vec3 pixel = vec3(uvs[0], uvs[1], 1.0);

	vec3 c = vec3(
		dot(pixel, vec3(p[0][0], p[1][0], p[3][0])) - b[0],
		dot(pixel, vec3(p[0][1], p[1][1], p[3][1])) - b[1],
		dot(pixel, vec3(p[0][2], p[1][2], p[3][2])) - b[2]
	);

	float denom = dot(s, p[2].xyz);

	float d = dot(s, c) / denom;
	float f = (f_comp(c, s, 0, 1) + f_comp(c, s, 1, 2) + f_comp(c, s, 0, 2)) / sq(denom);
	float r = sqrt(s[0] + s[1] + s[2]) / (f + 1);

	return k / r * (atan((d + 1) * r) - atan(d * r));
}

void main() {
	float sum = 0.0;
	for(int i = 0; i < NUM_SIGNALS; i++) {
		sum += integral(i);
	}

	sum = clamp(sum, unif_lower, unif_upper);
	sum = (sum - unif_lower) / (unif_upper - unif_lower);

	color = vec4(sum, sum, sum, 1.);
}
