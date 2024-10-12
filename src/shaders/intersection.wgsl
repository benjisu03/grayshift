
// STRUCTURES //

struct AABB {
	min: vec3<f32>,
	max: vec3<f32>
};

struct Ray {
	origin: vec3<f32>,
	direction: vec3<f32>,
	time: f32
};

struct RayInverse {
	origin: vec3<f32>,
	direction_inverse: vec3<f32>,
	time: f32
};

struct BVHNode {
	bbox: AABB,
	left: u32,
	right: u32,
	tri: u32
};

struct Triangle {
	a: vec3<f32>,
	b: vec3<f32>,
	c: vec3<f32>
};

struct TriangleIntersection {
	id: u32,
	t: f32,
	u: f32,
	v: f32
};

// INPUT DATA //

@group(0) @binding(0) var<storage, read> bvh: array<BVHNode>;
@group(0) @binding(1) var<storage, read> triangles: array<Triangle>;
@group(1) @binding(0) var<storage, read> rays: array<Ray>;


// OUTPUT DATA //

@group(1) @binding(1) var<storage, read_write> results: array<TriangleIntersection>;

// CONSTANTS //

const U32_MAX: u32 = 0xFFFFFFFFu;
const F32_MAX: f32 = 3.402823466e+38;
const STACK_LIMIT: u32 = 32;

// MAIN //
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
	let index = id.x;
	let ray = rays[index];

	results[index] = intersect_BVH(ray);
}

// FUNCTIONS //

fn intersect_BVH(ray: Ray) -> TriangleIntersection {
	var stack = array<u32, STACK_LIMIT>();
	var stack_size = 0;

	let direction_inverse = vec3(1.0 / ray.direction.x, 1.0 / ray.direction.y, 1.0 / ray.direction.z);
	let ray_inv = RayInverse(ray.origin, direction_inverse, ray.time);

	var result = TriangleIntersection(U32_MAX, 0.0, 0.0, 0.0);
	var ray_t_max = F32_MAX;

	// push root node
	let root_id = arrayLength(&bvh) - 1;
	stack[stack_size] = root_id;
	stack_size++;

	loop {
		if(stack_size == 0) {
            break;
        }

        stack_size--;
        let node_id = stack[stack_size];
        let node = bvh[node_id];

        if(!intersect_AABB(node.bbox, ray_inv, ray_t_max)) {
			continue;
		}

		if(node.tri != U32_MAX) {

			// leaf node
			let tri_hit = intersect_triangle(ray, node.tri, ray_t_max);

			if(tri_hit.t >= 0.0 && tri_hit.t <= ray_t_max) {
				result = tri_hit;
				ray_t_max = tri_hit.t;
			}

		} else {

			// branch node
			stack[stack_size] = node.left;
            stack_size++;

            stack[stack_size] = node.right;
            stack_size++;
		}
	}

	return result;
}


fn intersect_AABB(bbox: AABB, ray: RayInverse, ray_t_max: f32) -> bool {
	let t1 = (bbox.min - ray.origin) * ray.direction_inverse;
	let t2 = (bbox.max - ray.origin) * ray.direction_inverse;

	var tmin = min(t1.x, t2.x);
	var tmax = max(t1.x, t2.x);

	tmin = max(tmin, min(t1.y, t2.y));
	tmax = min(tmax, max(t1.y, t2.y));

	tmin = max(tmin, min(t1.z, t2.z));
	tmax = min(tmax, max(t1.z, t2.z));

	return tmax >= tmin && tmin > 0.0 && tmin < ray_t_max;
}

const EPSILON: f32 = 0.000001;

fn intersect_triangle(ray: Ray, triangle_id: u32, ray_max: f32) -> TriangleIntersection {
	let tri = triangles[triangle_id];

	let edge1 = tri.c - tri.a;
	let edge2 = tri.b - tri.a;

	let h = cross(ray.direction, edge2);
	let det = dot(edge1, h);

	if(det > -EPSILON && det < EPSILON) {
		return TriangleIntersection(U32_MAX, 0.0, 0.0, 0.0);
	}

	let inv_det = 1.0 / det;
	let s = ray.origin - tri.a;
	let u = inv_det * dot(s, h);

	if(u < 0.0 || u > 1.0) {
		return TriangleIntersection(U32_MAX, 0.0, 0.0, 0.0);
	}

	let q = cross(s, edge1);
	let v = inv_det * dot(ray.direction, q);

	if(v < 0.0 || u + v > 1.0) {
		return TriangleIntersection(U32_MAX, 0.0, 0.0, 0.0);
	}

	let t = inv_det * dot(edge2, q);

	return TriangleIntersection(triangle_id, t, u, v);
}