#[macro_use]
extern crate arrayref;

#[macro_use]
extern crate measure_time;

#[macro_use]
extern crate serde_derive;

mod transforms;

use std::error::Error;
use structopt::StructOpt;

use crate::transforms::gis::{Point3dd, Point3df};

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// position to transform
    #[structopt(long, short)]
    position: Vec<f64>,
}

#[derive(Serialize, Debug)]
struct ReqParam {
    source_space: String,
    target_space: String,
    source_points: Vec<Vec<f64>>,
}

#[derive(Deserialize, Debug)]
struct PointsResponse {
    target_points: Vec<Vec<f64>>,
}

const PRECISION: f64 = 1E6;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let opt = Opt::from_args();

    // 39d2e4cf-8979-9fb2-bc29-2a4ead14ae40 -> 23df7ce8-e405-bc31-3863-d543e3cc89e5
    //  => full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4.ima
    let basename = "full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4";
    let filename = format!("data/{}", basename);
    let gis = transforms::gis::load_file(&filename)?;

    // First retrieve reference implementation results:
    let client = reqwest::Client::new();

    // x y z
    //448 x 486 x 403

    let mut points = vec![];
    let mut my_results = vec![];
    for z in 0..10 {
        for y in 0..10 {
            for x in 0..10 {
                points.push(vec![x as f64, y as f64, z as f64]);
                my_results.push(gis.deformation(&(&vec![x as f64, y as f64, z as f64]).into()));
            }
        }
    }

    let param = ReqParam {
        source_space: "39d2e4cf-8979-9fb2-bc29-2a4ead14ae40".to_string(),
        target_space: "23df7ce8-e405-bc31-3863-d543e3cc89e5".to_string(),
        source_points: points.clone(),
    };

    let mut res = client
        .post("https://hbp-spatial-backend.apps-dev.hbp.eu/v1/transform-points")
        .json(&param)
        .send()?;
    let reference: PointsResponse = res.json()?;

    for i in 0..reference.target_points.len() {
        let r = &reference.target_points[i];
        let p = &my_results[i];

        if (r[0] - p[0]).abs() > PRECISION
            || (r[1] - p[1]).abs() > PRECISION
            || (r[2] - p[2]).abs() > PRECISION
        {
            println!("Mismatch r {:?} p {:?} ", r, p);
        }
    }

    /*
        // 39d2e4cf-8979-9fb2-bc29-2a4ead14ae40 -> 23df7ce8-e405-bc31-3863-d543e3cc89e5
        //  => full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4.ima
        let basename = "full_cls_400um_border_default_acquisition_disco_analysis_displ_field_DISCO_DARTEL_20181004_reg_x4";
        let filename = format!("data/{}", basename);
        let gis = transforms::gis::load_file(&filename)?;

        for z in 0..100 {
            for y in 0..100 {
                for x in 0..100 {
                    let r = reference_value(&res, x, y, z);
                    let p = gis.deformation((&vec![x, y, z]).into());

                    if (r[0] - p[0]) != 0.0 || (r[1] - p[1]) != 0.0 || (r[2] - p[2]) != 0.0 {
                        println!("Mismatch [{}, {}, {}] : r {:?} p {:?} ", x, y, z, r, p);
                    }
                }
            }
        }

        //let point = Point3dd([f64::from(100) * 1.0; 3]);
        let point = (&opt.position).into();
        //let Point3dd([a, b, c]) = point;

        let tx = gis.deformation(&point);
        let Point3dd([d, e, f]) = tx;

        println!("{:>3.16?},\t{:>3.16?},\t{:>3.16?}", d, e, f);
    */
    Ok(())
}
