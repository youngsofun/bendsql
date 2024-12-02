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

import org.junit.jupiter.api.Test;

import com.databend.bendsql.jni_utils.NativeLibrary;

import org.junit.jupiter.api.BeforeEach;
import static org.junit.jupiter.api.Assertions.*;

class NativeConnectionTest {
    private NativeConnection connection;
    private static final String TEST_DSN = "databend://root:@localhost:8000/default?sslmode=disable";

    static {
        NativeLibrary.loadLibrary();
    }

    
    @BeforeEach
    void setUp() {
        connection = NativeConnection.of(TEST_DSN);
    }

    @Test
    void testSimpleQuery() {
        String sql = "SELECT 1, 2";
        
        RowIterator result = connection.query(sql);
        
        assertNotNull(result);
        assertTrue(result.hasNext());
        Row row = result.next();
        assertEquals(1, row.getInt(0));
        assertEquals(2, row.getInt(1));
        // assertFalse(result.hasNext());
    }

    @Test
    void testQueryInvalidQuery() {
        String sql = "INVALID SQL QUERY";
        
        assertThrows(RuntimeException.class, () -> connection.query(sql));
    }

    @Test
    void testQueryNullQuery() {
        // 验证
        assertThrows(NullPointerException.class, () -> connection.query(null));
    }

    @Test
    void testQueryEmptyQuery() {
        // 准备
        String sql = "";
        
        // 验证
        assertThrows(RuntimeException.class, () -> connection.query(sql));
    }

    @Test
    void testMultipleQueriesSequentially() {
        // 准备
        String sql1 = "SELECT 1";
        String sql2 = "SELECT 2";
        
        // 执行
        RowIterator result1 = connection.query(sql1);
        RowIterator result2 = connection.query(sql2);
        
        // 验证
        assertNotNull(result1);
        assertNotNull(result2);
    }
} 