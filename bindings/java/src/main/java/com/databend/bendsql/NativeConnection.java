/*
 * Copyright 2021 Datafuse Labs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.databend.bendsql;


import java.util.HashSet;

import com.databend.bendsql.jni_utils.AsyncExecutor;
import com.databend.bendsql.jni_utils.NativeObject;

public class NativeConnection extends NativeObject {

    private final long executorHandle;
    private final HashSet<Long> resultHandles;

    public static NativeConnection of(String dsn) {
        return of(dsn, null);
    }

    public static NativeConnection of(String dsn, AsyncExecutor executor) {
        final long executorHandle = executor != null ? executor.getNativeHandle() : 0;
        final long nativeHandle = constructor(executorHandle, dsn);
        return new NativeConnection(nativeHandle, executorHandle);
    }

    private NativeConnection(long nativeHandle, long executorHandle) {
        super(nativeHandle);
        this.executorHandle = executorHandle;
        this.resultHandles = new HashSet<>();
    }

   
    public Long exec(String sql) {
        return exec(nativeHandle, executorHandle, sql);
    }

    public RowIterator query(String sql) {
        final long resultHandle = query(nativeHandle, executorHandle, sql);
        this.resultHandles.add(resultHandle);
        return new RowIterator(resultHandle, executorHandle, this);
    }

    public void close_result(long resultHandle) {
        this.resultHandles.remove(resultHandle);
    }

    @Override
    public void close() {
        super.close();
        for (Long resultHandle : resultHandles) {
            close_result(resultHandle);
        }
        this.resultHandles.clear();
    }

    @Override
    protected native void disposeInternal(long handle);

    private static native long constructor(long executorHandle, String dsn);
    private static native long exec(long nativeHandle, long executorHandle, String sql);
    private static native long query(long nativeHandle, long executorHandle, String sql);
}
