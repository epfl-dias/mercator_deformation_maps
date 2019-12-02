#[macro_use]
extern crate arrayref;

#[macro_use]
extern crate measure_time;

#[macro_use]
extern crate serde_derive;

mod nice_float;
mod transforms;

use std::error::Error;

use nice_float::NiceFloat;
use transforms::gis::GISTransform;
use transforms::gis::Point3dd;

const PRECISION: f64 = 1E6;

#[derive(Serialize, Debug)]
struct ReqParam {
    source_space: String,
    target_space: String,
    source_points: Vec<Vec<f64>>,
}

#[derive(Deserialize, Debug)]
struct PointsResponse {
    target_points: Vec<Vec<NiceFloat>>,
}

#[allow(clippy::many_single_char_names)]
fn compare_block(
    gis: &GISTransform,
    source_space: String,
    target_space: String,
    x: (i32, i32),
    y: (i32, i32),
    z: (i32, i32),
    step: f64,
) -> Result<(), Box<dyn Error>> {
    // First retrieve reference implementation results:
    let client = reqwest::Client::new();

    let mut source_points = vec![];
    let mut my_results = vec![];
    for z in z.0..z.1 {
        for y in y.0..y.1 {
            for x in x.0..x.1 {
                let p = vec![x as f64 * step, y as f64 * step, z as f64 * step];
                my_results.push(gis.deformation(&(&p).into()));
                source_points.push(p);
            }
        }
    }

    let param = ReqParam {
        source_space,
        target_space,
        source_points,
    };

    let mut res = client
        .post("https://hbp-spatial-backend.apps-dev.hbp.eu/v1/transform-points")
        .json(&param)
        .send()?;
    let unparsed = res.text()?;

    // NaN without quoting as part of flaoting point is not a valid number in the JSON standard.
    // We transform it to a valid string, and redefine a floating point ('NiceFloat') type which
    // process it.
    let normalized = unparsed.replace("NaN", r#""NaN""#);

    let reference: PointsResponse = match serde_json::from_str(&normalized) {
        Ok(v) => v,
        Err(e) => {
            println!("\n request: {:?}, error {:?}", normalized, e);
            return Err(Box::new(e));
        }
    };

    for (i, p) in my_results
        .iter()
        .enumerate()
        .take(reference.target_points.len())
    {
        let r: Point3dd = (&reference.target_points[i]
            .iter()
            .map(|nf| nf.into())
            .collect::<Vec<f64>>())
            .into();

        if (r[0] - p[0]).abs() > PRECISION
            || (r[1] - p[1]).abs() > PRECISION
            || (r[2] - p[2]).abs() > PRECISION
        {
            print!("\nMismatch r {:?} p {:?} ", r, p);
        } else {
            print!(".")
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    // x y z
    //448 x 486 x 403

    // 39d2e4cf-8979-9fb2-bc29-2a4ead14ae40 -> 23df7ce8-e405-bc31-3863-d543e3cc89e5
    //  => full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4.ima
    let source_space = "39d2e4cf-8979-9fb2-bc29-2a4ead14ae40".to_string();
    let target_space = "23df7ce8-e405-bc31-3863-d543e3cc89e5".to_string();
    let basename = "full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4";

    let filename = format!("data/{}", basename);
    let gis = transforms::gis::load_file(&filename)?;

    compare_block(&gis, source_space, target_space, (0, 10), (0, 10), (0, 10))?;

    println!("\nComparison finished.");

    Ok(())
}
