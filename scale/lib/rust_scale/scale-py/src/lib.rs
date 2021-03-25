use ndarray::parallel::prelude::*;
use ndarray::{s, Axis};
use numpy::{PyArray2, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::prelude::{pymodule, PyModule, PyResult, Python};
use pyo3::wrap_pyfunction;
use scale::lda;
use scale::snr;
use pyo3::types::PyList;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

fn str2method(s: &str) -> Result<ranklib::RankingMethod, &str> {
    match s {
        "naive" => Ok(ranklib::RankingMethod::Naive),
        "hist" => Ok(ranklib::RankingMethod::Hist),
        #[cfg(feature = "ntl")]
        "histbignum" => Ok(ranklib::RankingMethod::HistBigNum),
        #[cfg(feature = "hellib")]
        "hellib" => Ok(ranklib::RankingMethod::Hellib),
        _ => Err("Invalid method name"),
    }
}

#[pymodule]
fn scale(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<snr::SNR>()?;
    m.add_class::<lda::LDA>()?;
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    #[pyfn(m, "run_bp")]
    pub fn run_bp(
    py: Python,
    functions: &PyList,
    variables: &PyList,
    it: usize,
    vertex: usize,
    nc: usize,
    n: usize,
) -> PyResult<()> {
        scale::belief_propagation::run_bp(py,functions,variables,it,vertex,nc,n)
    }

    #[pyfn(m, "partial_cp")]
    fn partial_cp(
        _py: Python,
        traces: PyReadonlyArray2<i16>, // (len,N_sample)
        poi: PyReadonlyArray1<u32>,    // (Np,len)
        store: &PyArray2<i16>,
    ) {
        let traces = traces.as_array();
        let poi = poi.as_array();
        let mut store = unsafe { store.as_array_mut() };
        store
            .axis_iter_mut(Axis(1))
            .into_par_iter()
            .zip(poi.outer_iter().into_par_iter())
            .for_each(|(mut x, poi)| {
                let poi = poi.first().unwrap();
                x.assign(&traces.slice(s![.., *poi as usize]));
            });
    }

    #[pyfn(m, "rank_accuracy")]
    fn rank_accuracy(
        costs: Vec<Vec<f64>>,
        key: Vec<usize>,
        acc: f64,
        merge: Option<usize>,
        method: String,
    ) -> PyResult<(f64, f64, f64)> {
        let res = str2method(&method);
        match res {
            Ok(res) => {
                let res = res.rank_accuracy(&costs, &key, acc, merge);
                match res {
                    Ok(res) => Ok((res.min, res.est, res.max)),
                    Err(s) => {
                        println!("{}", s);
                        panic!()
                    }
                }
            }
            Err(_) => panic!(),
        }
        //return Ok((res.min, res.est, res.max));
    }

    #[pyfn(m, "rank_nbin")]
    fn rank_nbin(
        costs: Vec<Vec<f64>>,
        key: Vec<usize>,
        nb_bin: usize,
        merge: Option<usize>,
        method: String,
    ) -> PyResult<(f64, f64, f64)> {
        let res = str2method(&method);
        match res {
            Ok(res) => {
                let res = res.rank_nbin(&costs, &key, nb_bin, merge);
                match res {
                    Ok(res) => Ok((res.min, res.est, res.max)),
                    Err(s) => {
                        println!("{}", s);
                        panic!()
                    }
                }
            }
            Err(_) => panic!(),
        }
    }

    Ok(())
}