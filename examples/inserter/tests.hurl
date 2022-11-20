POST http://localhost:8787/test_do
{
    "insert": {
        "key": "hello",
        "value": "world!"
    }
}

POST http://localhost:8787/test_do
{
    "get": {
        "key": "hello"
    }
}