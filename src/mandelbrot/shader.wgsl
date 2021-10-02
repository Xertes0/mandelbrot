struct MandelbrotParameters {
	range_min: f64;
	range_max: f64;
	pos_x:     f64;
	pos_y:     f64;
    max_iter:  u32;
    width:     u32;
    height:    u32;
};

[[group(0), binding(0)]]
var<storage, read_write> v_i: u32;

[[group(0), binding(1)]]
var<storage, read_write> v_params: MandelbrotParameters;

fn map(val: f64, i_min: f64, i_max: f64, o_min: f64, o_max: f64) -> f64 {
    return (val-i_min)/(i_max-i_min) * (o_max-o_min) + o_min;
}

fn compute(params: MandelbrotParameters, i: u32) -> u32{
	var x1: u32 = i % params.width;
	var y1: u32 = i / params.height;
	var x: f64 = map(x1, 0., params.width , params.range_min, params.range_max) + params.pos_x;
	var y: f64 = map(y1, 0., params.height, params.range_min, params.range_max) + params.pos_y;

	var x2: f64 = 0.0f64;
	var y2: f64 = 0.0f64;

	var iter: u32 = 0u32;
	loop {
		if (x2*x2+y2*y2 <= 4.0f64 && iter < params.max_iter) {
			var x_new = x2*x2-y2*y2 + x;
			y2 = 2.0f64*x2*y2 +y;
			x2 = x_new;
			iter = iter + 1u32;
		} else {
			return iter;
		}
	}

	return iter;
}

[[stage(compute), workgroup_size(1)]]
fn main() {
    v_i = compute(v_params, v_i);
}
