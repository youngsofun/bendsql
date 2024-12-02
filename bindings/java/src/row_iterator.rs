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

use crate::jni_utils::executor::executor_or_default;
use crate::jni_utils::executor::Executor;
use crate::value::row_batch_to_java_2d_array;
use crate::Result;
use jni::objects::JClass;

use databend_driver::RowBatchIterator;
use jni::sys::{jlong, jobject};
use jni::JNIEnv;
use tokio_stream::StreamExt;

#[no_mangle]
pub extern "system" fn Java_com_databend_bendsql_RowIterator_fetchNextRowBatch(
    env: JNIEnv,
    _class: JClass,
    connection: *mut RowBatchIterator,
    executor: *const Executor,
) -> jobject {
    fetch_next_row_batch(env, connection, executor).unwrap_or_else(|_e| {
        // Log error if needed
        std::ptr::null_mut()
    })
}

fn fetch_next_row_batch(
    mut env: JNIEnv,
    it: *mut RowBatchIterator,
    executor: *const Executor,
) -> Result<jobject> {
    let mut it = unsafe { Box::from_raw(it) };

    let row_result =
        executor_or_default(&mut env, executor)?.block_on(async move { it.next().await });

    match row_result {
        Some(row_batch) => {
            eprintln!("row_batch: {:?}", row_batch);
            let row_batch = row_batch.unwrap();
            row_batch_to_java_2d_array(&mut env, row_batch)
        }
        None => Ok(std::ptr::null_mut()),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_databend_bendsql_RowIterator_disposeInternal(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let _ = unsafe { Box::from_raw(handle as *mut RowBatchIterator) };
    }
}
