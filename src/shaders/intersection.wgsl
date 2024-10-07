
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

struct IntersectionResult {
	node_id: u32,
	time: f32
};

struct BVHNode {
	bbox: AABB,
	left: u32,
	right: u32
};

// INPUT DATA //

@group(0) @binding(0) var<storage, read> bvh: array<BVHNode>;
@group(1) @binding(0) var<storage, read> rays: array<Ray>;


// OUTPUT DATA //

@group(1) @binding(1) var<storage, read_write> results: array<IntersectionResult>;


// CONSTANTS //

const U32_MAX: u32 = 0xFFFFFFFFu;
const STACK_SIZE: u32 = 32;

// MAIN //
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
	let index = id.x;
	let ray = rays[index];
	let ray_inv = RayInverse(ray.origin, 1.0 / ray.direction, ray.time);

	if(bvh[0].left == U32_MAX) {
		results[index] = IntersectionResult(0, ray.origin.z);
		return;
	}

	results[index] = intersect_BVH(ray_inv);
}

// FUNCTIONS //

fn intersect_BVH(ray: RayInverse) -> IntersectionResult {
	var stack = array<u32, STACK_SIZE>();
	var stack_size = 0;

	return IntersectionResult(0, ray.origin.z);
}


fn intersect_AABB(bbox: AABB, ray: RayInverse) -> IntersectionResult {
	let t1 = (bbox.min - ray.origin) * ray.direction_inverse;
	let t2 = (bbox.max - ray.origin) * ray.direction_inverse;

	var tmin = min(t1.x, t2.x);
	var tmax = max(t1.x, t2.x);

	tmin = max(tmin, min(t1.y, t2.y));
	tmax = min(tmax, max(t1.y, t2.y));

	tmin = max(tmin, min(t1.z, t2.z));
	tmax = min(tmax, max(t1.z, t2.z));

	return IntersectionResult(
		u32(tmax >= tmin && tmax >= 0.0 && tmin < ray.time),
		tmin
	);
}

//fn intersect_BVHNode(node: BVHNode, ray: RayInverse) -> IntersectionResult {
//	let node_hit = intersect_AABB(node.bbox, ray);
//	if(!node_hit.did_hit) { return IntersectionResult(false, 0.0); }
//
//	var left_hit = false;
//	if(node.left != U32_MAX) {
//		let left_node = bvh[node.left];
//        left_hit = intersect_BVHNode(left_node, ray);
//	}
//
//	var right_hit = false;
//    if(node.right != U32_MAX) {
//        let right_node = bvh[node.right];
//        right_hit = intersect_BVHNode(right_node, ray);
//    }
//
//	if(left_hit) {
//		if(right_hit) {
//			// both hit, find closer one
//			if(left_hit.time < right_hit.time) {
//				return left_hit;
//			} else {
//				return right_hit;
//			}
//		}
//
//		// only left hit
//		return left_hit;
//
//	} else if(right_hit) {
//		// only right hit
//		return right_hit;
//	}
//
//	// hit node but nothing inside
//	return IntersectionResult(false, 0.0);
//}
