import init_databend_wasm, {Connection} from '../pkg/databend_wasm.js';

const dsn = "databend://root:@localhost:8123?sslmode=disable&login=disable&max_rows_per_page=1"

let currentConnection;
let currentStatement;

async function startup() {
    await init_databend_wasm();
    const processButton = document.getElementById('processButton');
    processButton.addEventListener('click', handleButtonClick);
}


async function handleButtonClick() {
    const resultField = document.getElementById('resultField');
    try {
        const sql = document.getElementById('inputSQL').value;
        if (currentConnection == undefined) {
            currentConnection = Connection.new();
            await currentConnection.connect(dsn);
        }
        resultField.innerHTML = "";
        currentStatement = await currentConnection.execute(sql, handleQueyResult);
    } catch (error) {
        resultField.innerHTML = `${error}`;
    }
}

function handleQueyResult(s) {
    const resultField = document.getElementById('resultField');
    const jsonData = JSON.parse(s);
    resultField.innerHTML += `<pre>${JSON.stringify(jsonData, null, 2)}</pre><hr>`;
}

startup().catch(error => {
    console.error('初始化过程中发生错误:', error);
});
