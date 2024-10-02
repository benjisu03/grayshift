use std::sync::Arc;
use crate::hittable::BVH::BVHNode;
use crate::hittable::hittable::HittableList;
use crate::hittable::sphere::Sphere;
use crate::material::Lambertian;
use crate::util::vec3::Vec3;

fn intersection_test() {

    let material = Arc::new(Lambertian::from_color(Vec3::new(0.5, 0.5, 0.5)));

    let s1 = Box::new(Sphere::new_stationary(
        Vec3::new(0.0, 0.0, 0.0),
        1.0,
        material.clone()
    ));

    let s2 = Box::new(Sphere::new_stationary(
        Vec3::new(3.0, 0.0, 0.0),
        1.0,
        material.clone()
    ));

    let s3 = Box::new(Sphere::new_stationary(
        Vec3::new(5.0, 0.0, 0.0),
        1.0,
        material.clone()
    ));

    let mut world = HittableList::new();
    world.add(s1);
    world.add(s2);
    world.add(s3);

    let bvh = BVHNode::from_list(world);
}