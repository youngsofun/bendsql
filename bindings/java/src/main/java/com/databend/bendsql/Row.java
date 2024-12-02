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

public class Row {
    private Object[] data;

    public Row(Object[] rowData) {
        this.data = rowData;
    }

    public Object getValue(int columnIndex) {
        if (columnIndex >= data.length) {
            throw new IllegalArgumentException("Column index out of bounds");
        }
        return data[columnIndex];
    }

    public Integer getInt(int i) {
        Object value = getValue(i);
        
        if (value instanceof Integer) {
            return (Integer) value;
        } else if (value instanceof Long) {
            long longValue = (Long) value;
            // Check if the long value is within Integer range
            if (longValue >= Integer.MIN_VALUE && longValue <= Integer.MAX_VALUE) {
                return (int) longValue;
            }
            throw new IllegalArgumentException("Long value " + longValue + " cannot be converted to Integer");
        } else if (value instanceof Short) {
            return ((Short) value).intValue();
        } else if (value instanceof Byte) {
            return ((Byte) value).intValue();
        } else if (value instanceof String) {
            try {
                return Integer.parseInt((String) value);
            } catch (NumberFormatException e) {
                throw new IllegalArgumentException("Cannot convert String value '" + value + "' to Integer", e);
            }
        }
        
        throw new IllegalArgumentException("Cannot convert value of type " + 
            value.getClass().getName() + " to Integer");
    }

    public String getString(int i) {
        Object value = getValue(i);
        
        if (value == null) {
            return null;
        }
        
        // If it's already a String, return it directly
        if (value instanceof String) {
            return (String) value;
        }
        
        // Convert any other type to String using toString()
        return value.toString();
    }

    public Double getDouble(int i) {
        Object value = getValue(i);
        
        if (value instanceof Double) {
            return (Double) value;
        } else if (value instanceof Float) {
            return ((Float) value).doubleValue();
        } else if (value instanceof Integer) {
            return ((Integer) value).doubleValue();
        } else if (value instanceof Long) {
            return ((Long) value).doubleValue();
        } else if (value instanceof Short) {
            return ((Short) value).doubleValue();
        } else if (value instanceof Byte) {
            return ((Byte) value).doubleValue();
        } else if (value instanceof String) {
            try {
                return Double.parseDouble((String) value);
            } catch (NumberFormatException e) {
                throw new IllegalArgumentException("Cannot convert String value '" + value + "' to Double", e);
            }
        }
        
        throw new IllegalArgumentException("Cannot convert value of type " + 
            value.getClass().getName() + " to Double");
    }
} 