package com.databend.bendsql;

import java.io.Reader;
import java.sql.*;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import com.databend.client.QueryRowField;

public class DatabendResultSet extends AbstractDatabendResultSet {
    private final Statement statement;
    private final RowIterator iterator;
    private Row currentRow;
    private boolean isClosed;
    private final Map<String, Integer> fieldMap;

    public DatabendResultSet(Statement statement, List<QueryRowField> schema, RowIterator iterator) {
        super(Optional.of(statement), schema, null, "NotQueryResultSet");
        this.statement = statement;
        this.iterator = iterator;
        this.isClosed = false;
        this.fieldMap = new HashMap<>();
    }

    @Override
    public boolean next() throws SQLException {
        checkClosed();
        if (iterator.hasNext()) {
            currentRow = iterator.next();
            return true;
        }
        return false;
    }

    @Override
    public void close() throws SQLException {
        if (!isClosed) {
            iterator.close();
            isClosed = true;
        }
    }

    @Override
    public boolean isClosed() throws SQLException {
        return isClosed;
    }

    private void checkClosed() throws SQLException {
        if (isClosed) {
            throw new SQLException("ResultSet is closed");
        }
    }

    @Override
    public <T> T unwrap(Class<T> iface) throws SQLException {
        if (iface.isAssignableFrom(getClass())) {
            return iface.cast(this);
        }
        throw new SQLException("Cannot unwrap to " + iface.getName());
    }

    @Override
    public boolean isWrapperFor(Class<?> iface) throws SQLException {
        return iface.isAssignableFrom(getClass());
    }

}
