// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Result;
use databend_driver::{NumberValue, Value};
use databend_driver_core::rows::RowBatch;
use jni::objects::{JObject, JValue};
use jni::sys::{jboolean, jbyte, jdouble, jfloat, jint, jlong, jobject, jshort};
use jni::JNIEnv;

#[inline]
fn convert_value_to_java_object<'a>(value: &Value) -> Result<(&str, &str, Vec<JValue>)> {
    Ok(match value {
        Value::Null => ("java/lang/Object", "()V", vec![]),
        Value::Boolean(b) => (
            "java/lang/Boolean",
            "(Z)V",
            vec![JValue::Bool(*b as jboolean)],
        ),
        Value::Number(num) => match num {
            NumberValue::Int8(i) => ("java/lang/Byte", "(B)V", vec![JValue::Byte(*i as jbyte)]),
            NumberValue::Int16(i) => ("java/lang/Short", "(S)V", vec![JValue::Short(*i as jshort)]),
            NumberValue::Int32(i) => ("java/lang/Integer", "(I)V", vec![JValue::Int(*i as jint)]),
            NumberValue::Int64(i) => ("java/lang/Long", "(J)V", vec![JValue::Long(*i as jlong)]),
            NumberValue::UInt8(u) => ("java/lang/Integer", "(I)V", vec![JValue::Int(*u as jint)]),
            NumberValue::UInt16(u) => ("java/lang/Integer", "(I)V", vec![JValue::Int(*u as jint)]),
            NumberValue::UInt32(u) => ("java/lang/Long", "(J)V", vec![JValue::Long(*u as jlong)]),
            NumberValue::UInt64(u) => ("java/lang/Long", "(J)V", vec![JValue::Long(*u as jlong)]),
            NumberValue::Float32(f) => {
                ("java/lang/Float", "(F)V", vec![JValue::Float(*f as jfloat)])
            }
            NumberValue::Float64(f) => (
                "java/lang/Double",
                "(D)V",
                vec![JValue::Double(*f as jdouble)],
            ),
            _ => {
                return Err(
                    databend_driver::Error::Unexpected("Unsupported type".to_string()).into(),
                )
            }
        },
        Value::String(_) => ("java/lang/String", "()V", vec![]),
        _ => return Err(databend_driver::Error::Unexpected("Unsupported type".to_string()).into()),
    })
}

// todo(youngsofun): optimize this further   
pub(crate) fn row_batch_to_java_2d_array(env: &mut JNIEnv, row_batch: RowBatch) -> Result<jobject> {
    let num_cols = row_batch.columns.len();
    let array_class = env.find_class("[Ljava/lang/Object;")?;
    let java_col_batch = env.new_object_array(num_cols as i32, array_class, JObject::null())?;

    for (col_idx, column) in row_batch.columns.into_iter().enumerate() {
        let array_class = env.find_class("java/lang/Object")?;
        let col_length = column.len();
        let java_col = env.new_object_array(col_length as i32, array_class, JObject::null())?;

        for (row_idx, value) in column.into_iter().enumerate() {
            let java_value = convert_value_to_java_object(&value)?;
            let java_obj = env.new_object(java_value.0, java_value.1, &java_value.2)?;
            env.set_object_array_element(&java_col, row_idx as i32, java_obj)?;
        }

        env.set_object_array_element(&java_col_batch, col_idx as i32, java_col)?;
    }
    Ok(java_col_batch.into_raw())
}
