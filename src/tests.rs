use super::*;

use std::error::Error;

use log::info;
use log::trace;
use log::warn;
use serde::Deserialize;
use serde::Serialize;

use gis::GISTransform;
use nice_float::NiceFloat;
use transforms::point::Point3dd;

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

fn init() -> Result<(String, String, GISTransform), Box<dyn Error>> {
    match pretty_env_logger::try_init() {
        _ => (), // Just ignore whatever is going on...
    }

    // x y z
    //448 x 486 x 403

    // 39d2e4cf-8979-9fb2-bc29-2a4ead14ae40 -> 23df7ce8-e405-bc31-3863-d543e3cc89e5
    //  => full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4.ima
    let source_space = "39d2e4cf-8979-9fb2-bc29-2a4ead14ae40".to_string();
    let target_space = "23df7ce8-e405-bc31-3863-d543e3cc89e5".to_string();
    let basename = "full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4";

    let filename = format!("data/{}", basename);

    let gis = transforms::gis::load_file(&filename)?;

    info!(
        "dimensions: {:?}, {:?} [mm]",
        gis.dimensions(),
        gis.dimensions_mm()
    );
    Ok((source_space, target_space, gis))
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
) -> Result<bool, Box<dyn Error>> {
    fn values(start: i32, end: i32, delta: f64) -> Vec<f64> {
        let steps = ((end - start) as f64 / delta) as i32;
        let mut values = vec![];
        let mut v = start as f64;

        for _ in 0..steps {
            values.push(v);
            v += delta;
        }
        trace!("values {:?}", values);
        values
    }

    let mut mismatches = vec![];

    // First retrieve reference implementation results:
    let client = reqwest::Client::new();

    let mut source_points = vec![];
    let mut my_results = vec![];
    for z in values(z.0, z.1, step) {
        for y in values(y.0, y.1, step) {
            for x in values(x.0, x.1, step) {
                let p = vec![x, y, z];
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
            warn!("request: {:?}, error {:?}", normalized, e);
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
            warn!("Mismatch r {:?} p {:?} ", r, p);
            mismatches.push((r, p));
        } else {
            if p.is_nan() || r.is_nan() {
                warn!("Match   r {:?} p {:?} ", r, p);
            } else {
                trace!(".")
            }
        }
    }

    Ok(mismatches.len() == 0)
}

#[test]
fn check_diagonal() -> Result<(), Box<dyn Error>> {
    let (source_space, target_space, gis) = init()?;

    let mut success = true;
    let mut prev = 0;

    // The diagonal cannot go further than the smallest dimension.
    let max = gis.dimensions_mm()[0]
        .min(gis.dimensions_mm()[1])
        .min(gis.dimensions_mm()[2]) as i32;

    for d in (0..max).step_by(10) {
        info!(
            "Block [{}, {}, {}] - [{}, {}, {}]",
            prev, prev, prev, d, d, d,
        );

        success = compare_block(
            &gis,
            source_space.clone(),
            target_space.clone(),
            (prev, d),
            (prev, d),
            (prev, d),
            1.0,
        )? && success;

        prev = d;
    }

    info!("Comparison finished.");
    assert!(success);

    Ok(())
}

#[test]
fn check_interpolation_diagonal() -> Result<(), Box<dyn Error>> {
    let (source_space, target_space, gis) = init()?;

    let mut success = true;
    let mut prev = 0;

    // The diagonal cannot go further than the smallest dimension.
    let max = gis.dimensions_mm()[0]
        .min(gis.dimensions_mm()[1])
        .min(gis.dimensions_mm()[2]) as i32;

    for d in (0..max).step_by(10) {
        info!(
            "Block [{}, {}, {}] - [{}, {}, {}]",
            prev,
            prev,
            prev,
            prev + 1,
            prev + 1,
            prev + 1,
        );

        success = compare_block(
            &gis,
            source_space.clone(),
            target_space.clone(),
            (prev, prev + 1),
            (prev, prev + 1),
            (prev, prev + 1),
            0.1,
        )? && success;

        prev = d;
    }

    info!("\nComparison finished.");
    assert!(success);

    Ok(())
}

#[test]
#[ignore]
fn check_whole_space() -> Result<(), Box<dyn Error>> {
    let (source_space, target_space, gis) = init()?;

    let mut success = true;
    let (mut z_prev, mut y_prev, mut x_prev) = (0, 0, 0);
    let z_max = gis.dimensions_mm()[2] as i32;
    let y_max = gis.dimensions_mm()[1] as i32;
    let x_max = gis.dimensions_mm()[0] as i32;

    for z in (0..z_max).step_by(10) {
        for y in (0..y_max).step_by(10) {
            for x in (0..x_max).step_by(10) {
                info!(
                    "Block [{}, {}, {}] - [{}, {}, {}]",
                    x_prev, y_prev, z_prev, x, y, z
                );

                success = compare_block(
                    &gis,
                    source_space.clone(),
                    target_space.clone(),
                    (x_prev, x),
                    (y_prev, y),
                    (z_prev, z),
                    1.0,
                )? && success;

                x_prev = x;
            }
            y_prev = y;
        }
        z_prev = z;
    }

    info!("Comparison finished.");
    assert!(success);

    Ok(())
}

#[test]
fn check_nan() -> Result<(), Box<dyn Error>> {
    let (source_space, target_space, gis) = init()?;

    let (z_prev, y_prev, x_prev) = (20, 20, 20);
    let (z, y, x) = (21, 21, 21);

    info!(
        "Block [{}, {}, {}] - [{}, {}, {}]",
        x_prev, y_prev, z_prev, x, y, z
    );

    let success = compare_block(
        &gis,
        source_space.clone(),
        target_space.clone(),
        (x_prev, x),
        (y_prev, y),
        (z_prev, z),
        1.0,
    )?;

    assert!(success);

    Ok(())
}
