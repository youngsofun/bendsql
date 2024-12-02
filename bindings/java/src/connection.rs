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

use databend_driver::Connection;
use jni::objects::JClass;
use jni::objects::JString;
use jni::sys::jlong;
use jni::JNIEnv;

use crate::error::Error;
use crate::jni_utils::executor::executor_or_default;
use crate::jni_utils::executor::Executor;

use crate::jni_utils::jstring_to_string;
use crate::Result;

pub struct ConnectionWrapper {
    pub inner: Box<dyn Connection>,
}

impl ConnectionWrapper {
    pub fn new(inner: Box<dyn Connection>) -> Self {
        Self { inner }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_databend_bendsql_NativeConnection_constructor(
    mut env: JNIEnv,
    _: JClass,
    executor: *const Executor,
    dsn: JString,
) -> jlong {
    intern_constructor(&mut env, executor, dsn).unwrap_or_else(|e| {
        e.throw(&mut env);
        0
    })
}

fn intern_constructor(env: &mut JNIEnv, executor: *const Executor, dsn: JString) -> Result<jlong> {
    let dsn = jstring_to_string(env, &dsn)?;
    let result = executor_or_default(env, executor)?.block_on(async move {
        let conn =
            databend_driver::rest_api::RestAPIConnection::try_create(&dsn, "a_test".to_string())
                .await
                .map(|conn| ConnectionWrapper::new(Box::new(conn) as Box<dyn Connection>));
        let handle = conn
            .map(|conn| Box::into_raw(Box::new(conn)) as jlong)
            .map_err(|e| Error::from(e));
        handle
    })?;
    Ok(result)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_databend_bendsql_NativeConnection_exec(
    mut env: JNIEnv,
    _: JClass,
    connection: *mut ConnectionWrapper,
    executor: *const Executor,
    sql: JString,
) -> jlong {
    intern_exec(&mut env, connection, executor, sql).unwrap_or_else(|e| {
        e.throw(&mut env);
        0
    })
}

fn intern_exec(
    env: &mut JNIEnv,
    connection: *mut ConnectionWrapper,
    executor: *const Executor,
    sql: JString,
) -> Result<jlong> {
    let sql = jstring_to_string(env, &sql)?;
    let connection = unsafe { Box::from_raw(connection) };

    let result = executor_or_default(env, executor)?.block_on(async move {
        connection
            .inner
            .exec(&sql)
            .await
            .map(|result| Box::into_raw(Box::new(result)) as jlong)
            .map_err(Error::from)
    })?;
    Ok(result)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_databend_bendsql_NativeConnection_query(
    mut env: JNIEnv,
    _: JClass,
    connection: *mut ConnectionWrapper,
    executor: *const Executor,
    sql: JString,
) -> jlong {
    intern_query(&mut env, connection, executor, sql).unwrap_or_else(|e| {
        e.throw(&mut env);
        0
    })
}

fn intern_query(
    env: &mut JNIEnv,
    connection: *mut ConnectionWrapper,
    executor: *const Executor,
    sql: JString,
) -> Result<jlong> {
    let sql = jstring_to_string(env, &sql)?;
    let connection: Box<ConnectionWrapper> = unsafe { Box::from_raw(connection) };

    let it = executor_or_default(env, executor)?
        .block_on(async move { connection.inner.query_iter_batch(&sql).await })?;
    Ok(Box::into_raw(Box::new(it)) as jlong)
}
