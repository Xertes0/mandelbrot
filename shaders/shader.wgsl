[[block]]
struct MandelbrotParameters {
	range_min: f32;
	range_max: f32;
	pos_x:     f32;
	pos_y:     f32;
    max_iter:  u32;
    width:     u32;
    height:    u32;
};

[[block]]
struct Iters {
    data: array<array<u32,3>,1000000>;
};

[[group(0), binding(0)]]
var<storage, read_write> v_iters: Iters;

[[group(0), binding(1)]]
var<storage, read_write> v_params: MandelbrotParameters;

fn map(val: f32, i_min: f32, i_max: f32, o_min: f32, o_max: f32) -> f32 {
	return (val-i_min)/(i_max-i_min) * (o_max-o_min) + o_min;
}

fn compute(index: u32) -> array<u32,3>{
    var x1: u32 = index % v_params.width;
    var y1: u32 = index / v_params.height;

    var x: f32 = map(f32(x1), 0.0f32, f32(v_params.width),  v_params.range_min, v_params.range_max) + v_params.pos_x;
    var y: f32 = map(f32(y1), 0.0f32, f32(v_params.height), v_params.range_min, v_params.range_max) + v_params.pos_y;

    var x2: f32 = 0.0f32;
    var y2: f32 = 0.0f32;

    var iter: u32 = 0u32;
    loop {
        if (x2*x2+y2*y2 <= 4.0f32 && iter < v_params.max_iter) {
            var x_new = x2*x2-y2*y2 + x;
            y2 = 2.0f32*x2*y2 +y;
            x2 = x_new;
            iter = iter + 1u32;
        } else {
            break;
        }
    }

    //return iter;

    var normalized: f32 = map(f32(iter), 0.0f32, f32(v_params.max_iter), 0.0f32, 1.0f32);
    return array<u32,3>(
        u32((9.0f32*(1.0f32-normalized)*normalized*normalized*normalized*255.0f32)),
        u32((15.0f32*(1.0f32-normalized)*(1.0f32-normalized)*normalized*normalized*255.0f32)),
        u32((8.5f32*(1.0f32-normalized)*(1.0f32-normalized)*(1.0f32-normalized) * normalized*255.0f32))
    );
}

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    v_iters.data[global_id.x] = compute(global_id.x);
    //v_indices.data[123] = compute(global_id.x);
}
