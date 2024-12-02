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

import static org.assertj.core.api.Assertions.assertThat;

public class FieldTest {
    static {
        NativeLibrary.loadLibrary();
    }
    
    @Test
    public void testFieldCreation() {
        Field field = Field.test("test_name");
        assertThat(field).isNotNull();
        assertThat(field.getName()).isEqualTo("test_name"); // matches the value set in Rust
    }
} 