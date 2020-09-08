use crate::err::Error;

pub use serde_json::{json, Map};

use serde_json::Number;
use std::cmp::PartialOrd;
use std::mem;

pub type Json = serde_json::Value;
pub type JsonObj = Map<String, Json>;
pub type JsonNum = serde_json::Number;

pub fn count(val: &Json) -> Result<Json, Error> {
    Ok(json_count(val))
}

pub fn unique(val: &Json) -> Result<Json, Error> {
    Ok(json_unique(val))
}

trait Compare {
    fn gt(&self) -> bool;
    fn gte(&self) -> bool;
    fn lt(&self) -> bool;
    fn lte(&self) -> bool;
}

struct Cmp<T> {
    x: T,
    y: T,
}

impl<T: PartialOrd> Compare for Cmp<T> {
    fn gt(&self) -> bool {
        self.x > self.y
    }

    fn gte(&self) -> bool {
        self.x >= self.y
    }

    fn lt(&self) -> bool {
        self.x < self.y
    }

    fn lte(&self) -> bool {
        self.x <= self.y
    }
}

impl<T: PartialOrd> Cmp<T> {
    fn new(x: T, y: T) -> Cmp<T> {
        Self { x, y }
    }
}

trait GtLt {
    fn apply<T>(&self, x: T, y: T) -> bool
    where
        T: Ord;
}

struct Gt {}

impl GtLt for Gt {
    fn apply<T: Ord>(&self, x: T, y: T) -> bool {
        x > y
    }
}

struct Lt {}

impl GtLt for Lt {
    fn apply<T: Ord>(&self, x: T, y: T) -> bool {
        x < y
    }
}

pub fn json_eq(x: &Json, y: &Json) -> bool {
    x == y
}

pub fn json_neq(x: &Json, y: &Json) -> bool {
    x != y
}

fn json_cmp<'a>(x: &'a Json, y: &'a Json, p: &dyn Fn(&dyn Compare) -> bool) -> bool {
    match (x, y) {
        (Json::String(x), Json::String(y)) => {
            let cmp = Cmp::new(x, y);
            p(&cmp)
        }
        (Json::Number(x), Json::Number(y)) => match (x.is_i64(), y.is_i64()) {
            (true, true) => {
                let cmp = Cmp::new(x.as_i64().unwrap(), y.as_i64().unwrap());
                p(&cmp)
            }
            _ => {
                let cmp = Cmp::new(x.as_f64().unwrap(), y.as_f64().unwrap());
                p(&cmp)
            }
        },
        (Json::Bool(x), Json::Bool(y)) => {
            let cmp = Cmp::new(*x, *y);
            p(&cmp)
        }
        _ => false,
    }
}

pub fn json_gt(x: &Json, y: &Json) -> bool {
    json_cmp(x, y, &|x| x.gt())
}

pub fn json_lt(x: &Json, y: &Json) -> bool {
    json_cmp(x, y, &|x| x.lt())
}

pub fn json_gte(x: &Json, y: &Json) -> bool {
    json_cmp(x, y, &|x| x.gte())
}

pub fn json_lte(x: &Json, y: &Json) -> bool {
    json_cmp(x, y, &|x| x.lte())
}

pub fn json_count(val: &Json) -> Json {
    match val {
        Json::Array(ref arr) => Json::from(arr.len()),
        _ => Json::from(1),
    }
}

pub fn json_append(val: &mut Json, elem: Json) {
    match val {
        Json::Array(ref mut arr) => {
            arr.push(elem);
        }
        Json::Object(ref mut obj) => match elem {
            Json::Object(o) => {
                obj.extend(o);
            }
            _json => {
                let mut elem = Json::Array(Vec::new());
                mem::swap(&mut elem, val);
                json_append(val, elem);
            }
        },
        Json::Null => {
            *val = elem;
        }
        val => {
            let arr = vec![val.clone(), elem];
            *val = Json::from(arr);
        }
    }
}

pub fn json_first(val: &Json) -> &Json {
    match val {
        Json::Array(ref arr) if !arr.is_empty() => &arr[0],
        val => val,
    }
}

pub fn json_last(val: &Json) -> &Json {
    match val {
        Json::Array(ref arr) if !arr.is_empty() => &arr[arr.len() - 1],
        val => val,
    }
}

pub fn json_sum(val: &Json) -> Json {
    match val {
        Json::Number(val) => Json::Number(val.clone()),
        Json::Array(ref arr) => json_arr_sum(arr),
        _ => Json::Null,
    }
}

pub fn json_pop(val: &mut Json) -> Result<Option<Json>, Error> {
    match val {
        Json::Array(ref mut arr) => Ok(arr_pop(arr)),
        _ => Err(Error::ExpectedArr),
    }
}

fn arr_pop(arr: &mut Vec<Json>) -> Option<Json> {
    arr.pop()
}

pub fn json_avg(val: &Json) -> Result<Json, Error> {
    match val {
        Json::Number(val) => Ok(Json::Number(val.clone())),
        Json::Array(ref arr) => json_arr_avg(arr),
        _ => Err(Error::BadType),
    }
}

pub fn json_var(val: &Json) -> Result<Json, Error> {
    match val {
        Json::Number(val) => Ok(Json::Number(val.clone())),
        Json::Array(ref arr) => json_arr_var(arr),
        _ => Err(Error::BadType),
    }
}

pub fn json_dev(val: &Json) -> Result<Json, Error> {
    match val {
        Json::Number(val) => Ok(Json::Number(val.clone())),
        Json::Array(ref arr) => json_arr_dev(arr),
        _ => Err(Error::BadType),
    }
}

pub fn json_max(val: &Json) -> &Json {
    match val {
        Json::Array(ref arr) if !arr.is_empty() => arr_max(arr),
        val => val,
    }
}

//TODO(jaupe) add more cases
pub fn json_add(lhs: &Json, rhs: &Json) -> Result<Json, Error> {
    match (lhs, rhs) {
        (Json::Array(lhs), Json::Array(rhs)) => json_add_arrs(lhs, rhs),
        (Json::Array(lhs), Json::Number(rhs)) => json_add_arr_num(lhs, rhs),
        (Json::Number(rhs), Json::Array(lhs)) => json_add_arr_num(lhs, rhs),
        (Json::Number(lhs), Json::Number(rhs)) => Ok(Json::Number(json_add_nums(lhs, rhs))),
        (Json::String(lhs), Json::String(rhs)) => json_add_str(lhs, rhs),
        (Json::String(lhs), Json::Array(rhs)) => add_str_arr(lhs, rhs),
        (Json::Array(lhs), Json::String(rhs)) => add_arr_str(lhs, rhs),
        _ => Err(Error::BadType),
    }
}

//TODO(jaupe) add more cases
pub fn json_bar(lhs: &Json, rhs: &Json) -> Result<Json, Error> {
    match (lhs, rhs) {
        (Json::Array(lhs), Json::Array(rhs)) => json_bar_arrs(lhs, rhs),
        (Json::Array(lhs), rhs) => json_bar_arr_val(lhs, rhs),
        (Json::Number(lhs), Json::Number(rhs)) => json_bar_num_num(lhs, rhs),
        _ => Err(Error::BadType),
    }
}

fn json_bar_arrs(lhs: &[Json], rhs: &[Json]) -> Result<Json, Error> {
    let mut out = Vec::new();
    for (x, y) in lhs.iter().zip(rhs.iter()) {
        let val = json_bar(x, y)?;
        out.push(val);
    }
    Ok(Json::Array(out))
}

fn json_bar_arr_val(lhs: &[Json], rhs: &Json) -> Result<Json, Error> {
    let mut out = Vec::new();
    for val in lhs {
        let val = json_bar(val, rhs)?;
        out.push(val);
    }
    Ok(Json::Array(out))
}

fn json_bar_num_num(lhs: &Number, rhs: &Number) -> Result<Json, Error> {
    match (lhs.as_i64(), rhs.as_i64()) {
        (Some(x), Some(y)) => Ok(Json::from(x / y * y)),
        _ => Err(Error::BadType),
    }
}

pub fn json_sub(lhs: &Json, rhs: &Json) -> Result<Json, Error> {
    match (lhs, rhs) {
        (Json::Array(lhs), Json::Array(rhs)) => json_sub_arrs(lhs, rhs),
        (Json::Array(lhs), Json::Number(rhs)) => json_sub_arr_num(lhs, rhs),
        (Json::Number(lhs), Json::Array(rhs)) => json_sub_num_arr(lhs, rhs),
        (Json::Number(lhs), Json::Number(rhs)) => json_sub_nums(lhs, rhs),
        _ => Err(Error::BadType),
    }
}

pub fn json_mul(lhs: &Json, rhs: &Json) -> Result<Json, Error> {
    match (lhs, rhs) {
        (Json::Array(x), Json::Array(y)) => mul_arrs(x, y),
        (Json::Array(x), Json::Number(y)) => mul_arr_num(x, y),
        (Json::Number(x), Json::Array(y)) => mul_arr_num(y, x),
        (Json::Number(x), Json::Number(y)) => mul_nums(x, y),
        _ => Err(Error::BadType),
    }
}

pub fn json_unique(val: &Json) -> Json {
    match val {
        Json::Array(arr) => arr_unique(arr),
        val => val.clone(),
    }
}

fn arr_unique(arr: &[Json]) -> Json {
    let mut unique: Vec<Json> = Vec::new();
    for val in arr {
        let pos = unique.iter().find(|x| *x == val);
        if pos.is_none() {
            unique.push(val.clone());
        }
    }
    Json::Array(unique)
}

fn mul_vals(x: &Json, y: &Json) -> Result<Json, Error> {
    match (x, y) {
        (Json::Number(x), Json::Number(y)) => mul_nums(x, y),
        _ => Err(Error::BadType),
    }
}

fn mul_nums(x: &JsonNum, y: &JsonNum) -> Result<Json, Error> {
    let val = x.as_f64().unwrap() * y.as_f64().unwrap();
    Ok(Json::from(val))
}

fn mul_arr_num(x: &[Json], y: &JsonNum) -> Result<Json, Error> {
    let mut arr = Vec::new();
    for x in x.iter() {
        arr.push(mul_val_num(x, y)?);
    }
    Ok(Json::from(arr))
}

fn mul_val_num(x: &Json, y: &JsonNum) -> Result<Json, Error> {
    match x {
        Json::Number(ref x) => mul_nums(x, y),
        Json::Array(ref arr) => mul_arr_num(arr, y),
        _ => Err(Error::BadType),
    }
}

//TODO(jaupe) optimize by removing the temp allocs
fn mul_arrs(lhs: &[Json], rhs: &[Json]) -> Result<Json, Error> {
    let mut arr: Vec<Json> = Vec::new();
    for (x, y) in lhs.iter().zip(rhs.iter()) {
        arr.push(mul_vals(x, y)?);
    }
    Ok(Json::from(arr))
}

pub fn json_div(lhs: &Json, rhs: &Json) -> Result<Json, Error> {
    match (lhs, rhs) {
        (Json::Array(ref lhs), Json::Array(ref rhs)) => div_arrs(lhs, rhs),
        (Json::Array(ref lhs), Json::Number(ref rhs)) => div_arr_num(lhs, rhs),
        (Json::Number(ref lhs), Json::Array(ref rhs)) => div_num_arr(lhs, rhs),
        (Json::Number(ref lhs), Json::Number(ref rhs)) => div_nums(lhs, rhs),
        _ => Err(Error::BadType),
    }
}

fn div_nums(x: &JsonNum, y: &JsonNum) -> Result<Json, Error> {
    let val = x.as_f64().unwrap() / y.as_f64().unwrap();
    Ok(Json::from(val))
}

fn div_arrs(x: &[Json], y: &[Json]) -> Result<Json, Error> {
    let mut arr = Vec::new();
    for (x, y) in x.iter().zip(y.iter()) {
        arr.push(json_div(x, y)?);
    }
    Ok(Json::from(arr))
}

fn div_arr_num(x: &[Json], y: &JsonNum) -> Result<Json, Error> {
    let mut arr: Vec<Json> = Vec::new();
    for x in x {
        arr.push(div_val_num(x, y)?);
    }
    Ok(Json::from(arr))
}

fn div_val_num(x: &Json, y: &JsonNum) -> Result<Json, Error> {
    match x {
        Json::Number(ref x) => div_nums(x, y),
        Json::Array(ref x) => div_arr_num(x, y),
        _ => Err(Error::BadType),
    }
}

fn div_num_arr(x: &JsonNum, y: &[Json]) -> Result<Json, Error> {
    let mut arr: Vec<Json> = Vec::new();
    for y in y {
        arr.push(div_num_val(x, y)?);
    }
    Ok(Json::from(arr))
}

fn div_num_val(x: &JsonNum, y: &Json) -> Result<Json, Error> {
    match y {
        Json::Number(ref y) => div_nums(x, y),
        Json::Array(ref y) => div_num_arr(x, y),
        _ => Err(Error::BadType),
    }
}

fn json_add_str(x: &str, y: &str) -> Result<Json, Error> {
    let val = x.to_string() + y;
    Ok(Json::String(val))
}

fn add_str_arr(x: &str, y: &[Json]) -> Result<Json, Error> {
    let mut arr = Vec::with_capacity(y.len());
    for e in y {
        arr.push(add_str_val(x, e)?);
    }
    Ok(Json::Array(arr))
}

fn add_str_val(x: &str, y: &Json) -> Result<Json, Error> {
    match y {
        Json::String(y) => Ok(Json::from(x.to_string() + y)),
        _ => Err(Error::BadType),
    }
}

fn add_val_str(x: &Json, y: &str) -> Result<Json, Error> {
    match x {
        Json::String(x) => Ok(Json::from(x.to_string() + y)),
        _ => Err(Error::BadType),
    }
}

fn add_arr_str(lhs: &[Json], rhs: &str) -> Result<Json, Error> {
    let mut arr = Vec::with_capacity(lhs.len());
    for x in lhs {
        arr.push(add_val_str(x, rhs)?);
    }
    Ok(Json::Array(arr))
}

//TODO(jaupe) add better error handlinge
fn json_add_arr_num(x: &[Json], y: &JsonNum) -> Result<Json, Error> {
    let arr: Vec<Json> = x
        .iter()
        .map(|x| Json::from(x.as_f64().unwrap() + y.as_f64().unwrap()))
        .collect();
    Ok(Json::Array(arr))
}

fn json_add_arrs(lhs: &[Json], rhs: &[Json]) -> Result<Json, Error> {
    let vec = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(x, y)| json_add(x, y).unwrap())
        .collect();
    Ok(Json::Array(vec))
}

pub(crate) fn json_add_nums(x: &JsonNum, y: &JsonNum) -> JsonNum {
    match (x.is_i64(), y.is_i64()) {
        (true, true) => JsonNum::from(x.as_i64().unwrap() + y.as_i64().unwrap()),
        _ => JsonNum::from_f64(x.as_f64().unwrap() + y.as_f64().unwrap()).unwrap(),
    }
}

fn json_sub_arr_num(x: &[Json], y: &JsonNum) -> Result<Json, Error> {
    let arr = x
        .iter()
        .map(|x| Json::from(x.as_f64().unwrap() - y.as_f64().unwrap()))
        .collect();
    Ok(Json::Array(arr))
}

fn json_sub_num_arr(x: &JsonNum, y: &[Json]) -> Result<Json, Error> {
    let arr = y
        .iter()
        .map(|y| Json::from(x.as_f64().unwrap() - y.as_f64().unwrap()))
        .collect();
    Ok(Json::Array(arr))
}

fn json_sub_arrs(lhs: &[Json], rhs: &[Json]) -> Result<Json, Error> {
    let vec = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(x, y)| json_sub(x, y).unwrap())
        .collect();
    Ok(Json::Array(vec))
}

fn json_sub_nums(x: &JsonNum, y: &JsonNum) -> Result<Json, Error> {
    let val = x.as_f64().unwrap() - y.as_f64().unwrap();
    Ok(Json::from(val))
}

pub fn json_min(val: &Json) -> &Json {
    match val {
        Json::Array(ref arr) if !arr.is_empty() => arr_min(arr),
        val => val,
    }
}

fn json_arr_sum(s: &[Json]) -> Json {
    let mut total = JsonNum::from(0);
    for val in s {
        match val {
            Json::Number(num) => {
                total = json_add_nums(&total, num);
            }
            _ => continue,
        }
    }
    Json::Number(total)
}

pub fn json_obj_ref(val: &Json) -> Result<&JsonObj, Error> {
    match val {
        Json::Object(obj) => Ok(obj),
        _ => Err(Error::ExpectedObj),
    }
}

fn json_arr_avg(s: &[Json]) -> Result<Json, Error> {
    let mut total = 0.0f64;
    for val in s {
        total += json_f64(val).ok_or(Error::BadType)?;
    }
    let val = total / (s.len() as f64);
    let num = JsonNum::from_f64(val).ok_or(Error::BadType)?;
    Ok(Json::Number(num))
}

fn json_arr_var(s: &[Json]) -> Result<Json, Error> {
    let mut sum = 0.0f64;
    for val in s {
        sum += json_f64(val).ok_or(Error::BadType)?;
    }
    let mean = sum / ((s.len() - 1) as f64);
    let mut var = 0.0f64;
    for val in s {
        var += (json_f64(val).ok_or(Error::BadType)? - mean).powf(2.0);
    }
    var /= (s.len()) as f64;
    let num = JsonNum::from_f64(var).ok_or(Error::BadType)?;
    Ok(Json::Number(num))
}

fn json_arr_dev(s: &[Json]) -> Result<Json, Error> {
    let mut sum = 0.0f64;
    for val in s {
        sum += json_f64(val).ok_or(Error::BadType)?;
    }
    let avg = sum / (s.len() as f64);
    let mut var = 0.0f64;
    for val in s {
        var += (json_f64(val).ok_or(Error::BadType)? - avg).powf(2.0);
    }
    var /= s.len() as f64;
    let num = JsonNum::from_f64(var.sqrt()).ok_or(Error::BadType)?;
    Ok(Json::Number(num))
}

pub fn json_f64(val: &Json) -> Option<f64> {
    match val {
        Json::Number(num) => num.as_f64(),
        _ => None,
    }
}

fn arr_max(s: &[Json]) -> &Json {
    let mut max = &s[0];
    for val in &s[1..] {
        if json_gt(val, max) {
            max = val;
        }
    }
    max
}

fn arr_min(s: &[Json]) -> &Json {
    let mut min = &s[0];
    for val in &s[1..] {
        if json_lt(val, min) {
            min = val;
        }
    }
    min
}

pub fn json_string(x: &Json) -> Json {
    Json::String(x.to_string())
}

pub fn json_get<'a>(key: &str, val: &'a Json) -> Option<Json> {
    match val {
        Json::Array(arr) => {
            if arr.is_empty() {
                return None;
            }
            let mut out = Vec::new();
            for val in arr {
                if let Some(v) = json_get(key, val) {
                    out.push(v);
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(Json::Array(out))
            }
        }
        Json::Object(obj) => {
            if let Some(val) = obj.get(key) {
                Some(val.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn json_push(to: &mut Json, val: Json) {
    match to {
        Json::Array(ref mut arr) => {
            arr.push(val);
        }
        v => {
            //TODO optimize this to not clone the val
            *v = Json::from(vec![v.clone(), val])
        }
    };
}

pub fn json_insert(val: &mut Json, rows: Vec<JsonObj>) {
    match val {
        Json::Array(ref mut arr) => arr.extend(rows.into_iter().map(Json::Object)),
        val => {
            let mut arr = vec![val.clone()];
            arr.extend(rows.into_iter().map(Json::Object));
            *val = Json::Array(arr);
        }
    }
}

mod tests {

    use crate::json::*;

    #[test]
    fn append_obj_ok() {
        use serde_json::json;
        let mut obj = json!({"name":"james"});
        let elem = json!({"age": 45});
        json_append(&mut obj, elem);
        assert_eq!(obj, json!({"name": "james", "age": 45}));
    }

    #[test]
    fn json_append_obj() {
        let mut obj = json!({"name":"anna", "age": 28});
        json_append(&mut obj, json!({"email": "anna@gmail.com"}));
        assert_eq!(
            obj,
            json!({"name": "anna", "age": 28, "email": "anna@gmail.com"})
        )
    }

    #[test]
    fn json_get_arr_obj() {
        let obj = json!({"name":"anna", "age": 28});
        assert_eq!(Some(Json::from("anna")), json_get("name", &obj));
        assert_eq!(Some(Json::from(28)), json_get("age", &obj));
    }

    #[test]
    fn json_insert_arr() {
        let f = |x: Json| x.as_object().cloned().unwrap();
        let mut val = json!([{"name":"anna", "age": 28}]);
        json_insert(
            &mut val,
            vec![
                f(json!({"name":"james", "age": 32})),
                f(json!({"name":"misha", "age": 9})),
            ],
        );
        assert_eq!(
            json!([
                {"name":"anna", "age": 28},
                {"name":"james", "age": 32},
                {"name":"misha", "age": 9},
            ]),
            val
        );
    }

    #[test]
    fn json_bar_ok() {
        let lhs = json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let rhs = Json::from(2);
        assert_eq!(
            Ok(json!([0, 0, 2, 2, 4, 4, 6, 6, 8, 8])),
            json_bar(&lhs, &rhs)
        );
    }
}
