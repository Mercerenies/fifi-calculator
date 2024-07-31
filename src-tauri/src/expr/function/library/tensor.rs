
//! Functions which operate on vectors.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::vector::{Vector, vector_shape};
use crate::expr::vector::tensor::Tensor;
use crate::expr::prisms;
use crate::expr::ordering::cmp_expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::algebra::infinity::InfiniteConstant;
use crate::util::{repeated, clamp};
use crate::util::prism::Identity;

use num::BigInt;
use itertools::Itertools;

use std::cmp::Ordering;

pub fn append_tensor_functions(table: &mut FunctionTable) {
  table.insert(vconcat());
  table.insert(repeat());
  table.insert(iota());
  table.insert(head());
  table.insert(tail());
  table.insert(last());
  table.insert(init());
  table.insert(cons());
  table.insert(snoc());
  table.insert(nth());
  table.insert(remove_nth());
  table.insert(nth_column());
  table.insert(remove_nth_column());
  table.insert(subvector());
  table.insert(remove_subvector());
  table.insert(vec_length());
  table.insert(vec_shape());
  table.insert(find_in_vector());
  table.insert(arrange_vector());
  table.insert(sort_vector());
  table.insert(sort_vector_reversed());
}

fn is_empty_vector(expr: &Expr) -> bool {
  expr == &Expr::from(Vector::empty())
}

pub fn vconcat() -> Function {
  FunctionBuilder::new("vconcat")
    .permit_flattening()
    .set_identity(is_empty_vector)
    .add_case(
      // Vector concatenation
      builder::any_arity().of_type(prisms::ExprToTensor).and_then(|args, _| {
        let sum = args.into_iter()
          .map(Tensor::into_vector)
          .fold(Vector::empty(), Vector::append);
        Ok(sum.into())
      })
    )
    .add_case(
      // String concatenation
      builder::any_arity().of_type(prisms::expr_to_string()).and_then(|args, _| {
        Ok(Expr::from(args.join("")))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative, similar to vectors.
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("vconcat", args))
      }
    )
    .build()
}

pub fn repeat() -> Function {
  FunctionBuilder::new("repeat")
    .add_case(
      builder::arity_two().of_types(Identity, prisms::expr_to_usize()).and_then(|value, len, _| {
        let vector: Vector = repeated(value, len);
        Ok(vector.into())
      })
    )
    .build()
}

pub fn iota() -> Function {
  FunctionBuilder::new("iota")
    .add_case(
      builder::arity_one().of_type(prisms::expr_to_i64()).and_then(|arg, _| {
        let vector: Vector = {
          match arg.cmp(&0) {
            Ordering::Greater => {
              (1..=arg).map(Expr::from).collect()
            }
            Ordering::Less => {
              (arg..=-1).map(Expr::from).collect()
            }
            Ordering::Equal => {
              Vector::empty()
            }
          }
        };
        Ok(vector.into())
      })
    )
    .build()
}

pub fn head() -> Function {
  FunctionBuilder::new("head")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("head", "head called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          Ok(vec.swap_remove(0)) // bounds safety: We just checked if the vec was empty
        }
      })
    )
    .build()
}

pub fn tail() -> Function {
  FunctionBuilder::new("tail")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("tail", "tail called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          vec.remove(0); // bounds safety: We just checked if the vec was empty
          Ok(Vector::from(vec).into())
        }
      })
    )
    .build()
}

pub fn last() -> Function {
  FunctionBuilder::new("last")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("last", "last called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          Ok(vec.swap_remove(vec.len() - 1)) // bounds safety: We just checked if the vec was empty
        }
      })
    )
    .build()
}

pub fn init() -> Function {
  FunctionBuilder::new("init")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("init", "init called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          vec.pop().unwrap(); // unwrap safety: We just checked if the vec was empty
          Ok(Vector::from(vec).into())
        }
      })
    )
    .build()
}

pub fn cons() -> Function {
  FunctionBuilder::new("cons")
    .add_case(
      builder::arity_two().of_types(Identity, prisms::ExprToVector).and_then(|new_value, mut vec, _| {
        vec.as_mut_vec().insert(0, new_value);
        Ok(vec.into())
      })
    )
    .build()
}

pub fn snoc() -> Function {
  FunctionBuilder::new("snoc")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, Identity).and_then(|mut vec, new_value, _| {
        vec.as_mut_vec().push(new_value);
        Ok(vec.into())
      })
    )
    .build()
}

pub fn nth() -> Function {
  FunctionBuilder::new("nth")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, prisms::expr_to_i64()).and_then(|mut vec, index, ctx| {
        let unsigned_index =
          if index < - (vec.len() as i64) {
            ctx.errors.push(SimplifierError::custom_error("nth", "Index out of bounds"));
            return Err((vec, index));
          } else if index < 0 {
            vec.len() - (-index) as usize
          } else {
            index as usize
          };
        if vec.get(unsigned_index).is_some() {
          // We're about to drop the vector, so we can safely remove
          // things from it.
          Ok(vec.as_mut_vec().swap_remove(unsigned_index))
        } else {
          ctx.errors.push(SimplifierError::custom_error("nth", "Index out of bounds"));
          Err((vec, index))
        }
      })
    )
    .build()
}

pub fn remove_nth() -> Function {
  FunctionBuilder::new("remove_nth")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, prisms::expr_to_i64()).and_then(|mut vec, index, ctx| {
        let unsigned_index =
          if index < - (vec.len() as i64) {
            ctx.errors.push(SimplifierError::custom_error("nth", "Index out of bounds"));
            return Err((vec, index));
          } else if index < 0 {
            vec.len() - (-index) as usize
          } else {
            index as usize
          };
        if vec.get(unsigned_index).is_some() {
          vec.as_mut_vec().remove(unsigned_index);
          Ok(vec.into())
        } else {
          ctx.errors.push(SimplifierError::custom_error("nth", "Index out of bounds"));
          Err((vec, index))
        }
      })
    )
    .build()
}

pub fn nth_column() -> Function {
  FunctionBuilder::new("nth_column")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToMatrix, prisms::expr_to_i64()).and_then(|mat, index, ctx| {
        let unsigned_index =
          if index < - (mat.width() as i64) {
            ctx.errors.push(SimplifierError::custom_error("nth_column", "Index out of bounds"));
            return Err((mat, index));
          } else if index < 0 {
            mat.width() - (-index) as usize
          } else {
            index as usize
          };
        match mat.column(unsigned_index) {
          Some(column) => {
            let vec = Vector::from(column.to_owned());
            Ok(Expr::from(vec))
          }
          None => {
            ctx.errors.push(SimplifierError::custom_error("nth_column", "Index out of bounds"));
            Err((mat, index))
          }
        }
      })
    )
    .build()
}

pub fn remove_nth_column() -> Function {
  FunctionBuilder::new("remove_nth_column")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToMatrix, prisms::expr_to_i64()).and_then(|mut mat, index, ctx| {
        let unsigned_index =
          if index < - (mat.width() as i64) {
            ctx.errors.push(SimplifierError::custom_error("remove_nth_column", "Index out of bounds"));
            return Err((mat, index));
          } else if index < 0 {
            mat.width() - (-index) as usize
          } else {
            index as usize
          };
        match mat.as_matrix_mut().remove_column(unsigned_index) {
          Some(_) => {
            Ok(Expr::from(mat))
          }
          None => {
            ctx.errors.push(SimplifierError::custom_error("remove_nth_column", "Index out of bounds"));
            Err((mat, index))
          }
        }
      })
    )
    .build()
}

pub fn subvector() -> Function {
  FunctionBuilder::new("subvector")
    .add_case(
      builder::arity_three().of_types(prisms::ExprToVector, prisms::expr_to_i64(), prisms::expr_to_i64())
        .and_then(|mut vec, start, end, _| {
          let subvector = extract_subvector(vec.as_mut_vec(), start, end);
          Ok(Expr::from(Vector::from(subvector)))
        })
    )
    .add_case(
      builder::arity_three().of_types(prisms::ExprToVector, prisms::expr_to_i64(), prisms::ExprToInfinity)
        .and_then(|mut vec, start, end_inf, ctx| {
          if end_inf != InfiniteConstant::PosInfinity {
            ctx.errors.push(SimplifierError::custom_error("subvector", "Expected positive infinity"));
            return Err((vec, start, end_inf));
          }
          let subvector = extract_subvector(vec.as_mut_vec(), start, i64::MAX);
          Ok(Expr::from(Vector::from(subvector)))
        })
    )
    .build()
}

pub fn remove_subvector() -> Function {
  FunctionBuilder::new("remove_subvector")
    .add_case(
      builder::arity_three().of_types(prisms::ExprToVector, prisms::expr_to_i64(), prisms::expr_to_i64())
        .and_then(|mut vec, start, end, _| {
          let _subvector = extract_subvector(vec.as_mut_vec(), start, end);
          Ok(Expr::from(vec))
        })
    )
    .add_case(
      builder::arity_three().of_types(prisms::ExprToVector, prisms::expr_to_i64(), prisms::ExprToInfinity)
        .and_then(|mut vec, start, end_inf, ctx| {
          if end_inf != InfiniteConstant::PosInfinity {
            ctx.errors.push(SimplifierError::custom_error("subvector", "Expected positive infinity"));
            return Err((vec, start, end_inf));
          }
          let _subvector = extract_subvector(vec.as_mut_vec(), start, i64::MAX);
          Ok(Expr::from(vec))
        })
    )
    .build()
}

fn extract_subvector(vector: &mut Vec<Expr>, mut start: i64, mut end: i64) -> Vec<Expr> {
  if start < 0 {
    start += vector.len() as i64;
  }
  if end < 0 {
    end += vector.len() as i64;
  }
  let start = clamp(start, 0, vector.len() as i64) as usize;
  let end = clamp(end, 0, vector.len() as i64) as usize;
  vector.drain(start..end).collect()
}

pub fn vec_length() -> Function {
  FunctionBuilder::new("length")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, _| {
        let length = vec.len();
        Ok(Expr::from(BigInt::from(length)))
      })
    )
    .build()
}

pub fn vec_shape() -> Function {
  FunctionBuilder::new("shape")
    .add_case(
      builder::arity_one().of_type(Identity).and_then(|value, _| {
        let shape = vector_shape(&value);
        let shape_as_vec: Vector = shape.into_iter().map(|x| Expr::from(BigInt::from(x))).collect();
        Ok(shape_as_vec.into())
      })
    )
    .build()
}

pub fn find_in_vector() -> Function {
  FunctionBuilder::new("find")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, Identity).and_then(|haystack, needle, _| {
        let index = haystack.iter().position(|x| *x == needle).map_or(-1, |i| i as i64);
        Ok(Expr::from(index))
      })
    )
    .build()
}

pub fn arrange_vector() -> Function {
  FunctionBuilder::new("arrange")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, prisms::expr_to_usize()).and_then(|vector, chunk_size, _| {
        let vector = vector.flatten_all_nested();
        if chunk_size == 0 {
          return Ok(Expr::from(vector));
        }
        let chunked_vector = vector.into_iter()
          .chunks(chunk_size)
          .into_iter()
          .map(|chunk| Expr::from(chunk.into_iter().collect::<Vector>()))
          .collect::<Vector>();
        Ok(Expr::from(chunked_vector))
      })
    )
    .build()
}

pub fn sort_vector() -> Function {
  FunctionBuilder::new("sort")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|mut vec, _| {
        vec.as_mut_slice().sort_by(cmp_expr);
        Ok(Expr::from(vec))
      })
    )
    .build()
}

pub fn sort_vector_reversed() -> Function {
  FunctionBuilder::new("rsort")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|mut vec, _| {
        vec.as_mut_slice().sort_by(|a, b| cmp_expr(b, a));
        Ok(Expr::from(vec))
      })
    )
    .build()
}
