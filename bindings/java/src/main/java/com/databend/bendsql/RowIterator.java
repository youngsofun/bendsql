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

import java.util.Iterator;
import java.util.NoSuchElementException;

import com.databend.bendsql.jni_utils.NativeObject;

public class RowIterator extends NativeObject implements Iterator<Row> {
    private final long connectionHandle;
    private final long executorHandle;
    private final NativeConnection connection;

    private Object[][] columns;
    private int currentIndex;
    private boolean isFinished;

    public RowIterator(long nativeHandle, long executorHandle, NativeConnection connection) {
        super(nativeHandle);
        this.connectionHandle = nativeHandle;
        this.executorHandle = executorHandle;
        this.connection = connection;
        this.currentIndex = 0;
        this.isFinished = false;
        this.columns = fetchNextRowBatch(nativeHandle, executorHandle);
        if (this.columns == null || this.columns.length == 0 || this.columns[0].length == 0) {
            this.isFinished = true;
        }
    }

    @Override
    public boolean hasNext() {
        if (columns != null && columns.length > 0 && currentIndex < columns[0].length) {
            return true;
        }
        if (isFinished) {
            return false;
        }
        columns = fetchNextRowBatch(nativeHandle, executorHandle);
        currentIndex = 0;
        if (columns == null || columns.length == 0 || columns[0].length == 0) {
            isFinished = true;
            return false;
        }
        return true;
    }

    @Override
    public void close() {
        super.close();
        connection.close_result(nativeHandle);
    }

    @Override
    public Row next() {
        if (!hasNext()) {
            throw new NoSuchElementException();
        }
        Object[] array = new Object[columns.length];
        for (int i = 0; i < columns.length; i++) {
            array[i] = columns[i][currentIndex];
        }
        currentIndex++;
        return new Row(array);
    }

    private native Object[][] fetchNextRowBatch(long nativeHandle, long executorHandle);

    protected native void disposeInternal(long handle);
}
