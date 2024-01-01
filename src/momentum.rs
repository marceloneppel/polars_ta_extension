use crate::utils::get_series_f64_ptr;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;
use talib_sys::{TA_ADX_Lookback, TA_Integer, TA_Real, TA_RetCode, TA_ADX};
use talib_sys::{TA_AROON_Lookback, TA_AROON};
#[derive(Deserialize)]
pub struct ADXKwargs {
    timeperiod: i32,
}

#[polars_expr(output_type=Float64)]
fn adx(inputs: &[Series], kwargs: ADXKwargs) -> PolarsResult<Series> {
    let high = &mut inputs[1].to_float()?.rechunk();
    let low = &mut inputs[2].to_float()?.rechunk();
    let close = &mut inputs[0].to_float()?.rechunk();
    let mut out_begin: TA_Integer = 0;
    let mut out_size: TA_Integer = 0;
    let len = close.len();
    let mut out: Vec<TA_Real> = Vec::with_capacity(len);
    let (high_ptr, _high) = get_series_f64_ptr(high)?;
    let (low_ptr, _low) = get_series_f64_ptr(low)?;
    let (close_ptr, _close) = get_series_f64_ptr(close)?;

    let lookback = unsafe { TA_ADX_Lookback(kwargs.timeperiod) as usize };
    for _ in 0..lookback {
        out.push(std::f64::NAN);
    }
    let ret_code = unsafe {
        TA_ADX(
            0,
            len as i32 - 1,
            high_ptr,
            low_ptr,
            close_ptr,
            kwargs.timeperiod,
            &mut out_begin,
            &mut out_size,
            out[lookback..].as_mut_ptr(),
        )
    };
    match ret_code {
        TA_RetCode::TA_SUCCESS => {
            let out_size = (out_begin + out_size) as usize;
            if out_size != 0 {
                unsafe {
                    out.set_len(out_size);
                }
            } else {
                unsafe {
                    out.set_len(len);
                }
            }
            let out_ser = Float64Chunked::from_vec("", out);
            Ok(out_ser.into_series())
        }
        _ => Err(PolarsError::ComputeError(
            format!("Could not compute indicator, err: {:?}", ret_code).into(),
        )),
    }
}

#[derive(Deserialize)]
pub struct ArronKwargs {
    timeperiod: i32,
}

pub fn arron_output(_: &[Field]) -> PolarsResult<Field> {
    let d = Field::new("aroondown", DataType::Float64);
    let u = Field::new("aroonup", DataType::Float64);
    let v: Vec<Field> = vec![d, u];
    Ok(Field::new("", DataType::Struct(v)))
}


#[polars_expr(output_type_func=arron_output)]
fn aroon(inputs: &[Series], kwargs: ArronKwargs) -> PolarsResult<Series> {
    let high = &mut inputs[0].to_float()?.rechunk();
    let low = &mut inputs[1].to_float()?.rechunk();
    let len = high.len();
    let mut out_begin: TA_Integer = 0;
    let mut out_size: TA_Integer = 0;
    let mut outaroondown: Vec<TA_Real> = Vec::with_capacity(len);
    let mut outaroonup: Vec<TA_Real> = Vec::with_capacity(len);
    
    let (high_ptr, _high) = get_series_f64_ptr(high)?;
    let (low_ptr, _low) = get_series_f64_ptr(low)?;
    let lookback = unsafe {
        TA_AROON_Lookback(
            kwargs.timeperiod,
        ) as usize
    };
    for _ in 0..lookback {
        outaroondown.push(std::f64::NAN);
        outaroonup.push(std::f64::NAN);
    }
    let ret_code = unsafe {
        TA_AROON(
            0,
            len as i32 - 1,
            high_ptr,
            low_ptr,
            kwargs.timeperiod,
            &mut out_begin,
            &mut out_size,
            outaroondown[lookback..].as_mut_ptr(),
            outaroonup[lookback..].as_mut_ptr(),
        )
    };
    match ret_code {
        TA_RetCode::TA_SUCCESS => {
            let out_size = (out_begin + out_size) as usize;
            if out_size != 0 {
                unsafe {
                    outaroondown.set_len(out_size);
                    outaroonup.set_len(out_size);
                }
            } else {
                unsafe {
                    outaroondown.set_len(len);
                    outaroonup.set_len(len);
                }
            }
            let d = Series::from_vec("aroondown", outaroondown);
            let u = Series::from_vec("aroonup", outaroonup);
            let out = StructChunked::new("", &[d, u])?;
            Ok(out.into_series())
        }
        _ => Err(PolarsError::ComputeError(
            format!("Could not compute indicator, err: {:?}", ret_code).into(),
        )),
    }
}