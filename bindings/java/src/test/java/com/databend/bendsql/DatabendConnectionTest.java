package com.databend.bendsql;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import java.sql.ResultSet;
import java.sql.Statement;
import java.util.Properties;

import static org.junit.jupiter.api.Assertions.*;

public class DatabendConnectionTest {
    
    private DatabendConnection connection;
    private static final String TEST_DSN = "databend://root:@localhost:8000/default?sslmode=disable";
    
    @BeforeEach
    void setUp() throws Exception {
        Properties props = new Properties();
        connection = new DatabendConnection(TEST_DSN, props);
    }
    
    @Test
    void testCreateStatement() throws Exception {
        Statement stmt = connection.createStatement();
        assertNotNull(stmt);
        assertTrue(stmt instanceof DatabendStatement);
    }
    
    @Test
    void testSimpleQuery() throws Exception {
        Statement stmt = connection.createStatement();
        
        // 执行简单的 SELECT 查询
        ResultSet rs = stmt.executeQuery("SELECT 1");
        
        assertTrue(rs.next());
        assertEquals(1, rs.getInt(1));
        assertFalse(rs.next());
    }
    
    @Test
    void testTableQuery() throws Exception {
        Statement stmt = connection.createStatement();
        
        // 创建测试表
        stmt.execute("CREATE TABLE IF NOT EXISTS test_table (id INT, name VARCHAR)");
        
        // 插入测试数据
        stmt.execute("INSERT INTO test_table VALUES (1, 'test')");
        
        // 查询数据
        ResultSet rs = stmt.executeQuery("SELECT * FROM test_table");
        
        assertTrue(rs.next());
        assertEquals(1, rs.getInt("id"));
        assertEquals("test", rs.getString("name"));
        assertFalse(rs.next());
        
        // 清理测试数据
        stmt.execute("DROP TABLE IF EXISTS test_table");
    }
} 