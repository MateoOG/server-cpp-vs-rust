#same just tell me what to modify so we save your resources, dont produce all the artifact:

echo "Create task"
curl -X POST http://localhost:5000/task/create \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-002",
    "title": "Calculate factorial",
    "priority": 2,
    "data": {
      "type": "calculation", 
      "input": 20,
      "operation": "factorial"
    }
  }'
echo "Created task"

echo "Create task"
curl -X POST http://localhost:5000/task/create \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-003",
    "title": "Calculate factorial",
    "priority": 2,
    "data": {
      "type": "calculation", 
      "input": 20,
      "operation": "factorial"
    },
    "status": "pending"
  }'
echo "Created task"

echo "Create task"
curl -X POST http://localhost:5000/task/create \
  -H "Content-Type: application/json" \
  -d '{"id":"test-004","title":"Calculate factorial","priority":2,"data":{"type":"calculation","input":20,"operation":"factorial"},"status": "pending"}'
echo "Created task"

echo "Get task"
curl -X GET http://localhost:5000/task/test-002
echo "Got task"
echo "Get task"
curl -X GET http://localhost:5000/task/test-003
echo "Got task"
echo "Get task"
curl -X GET http://localhost:5000/task/test-004
echo "Got task"

echo "Complete task"
curl -X POST http://localhost:5000/task/test-003/complete \
  -H "Content-Type: application/json" \
  -d ""
echo "Completed task"
echo "Complete task"
curl -X POST http://localhost:5000/task/test-002/complete \
  -H "Content-Type: application/json" \
  -d ""
echo "Completed task"
echo "Complete task"
curl -X POST http://localhost:5000/task/test-004/complete \
  -H "Content-Type: application/json" \
  -d ""
echo "Completed task"
echo "Get task"
curl -X GET http://localhost:5000/task/test-002
echo "Got task"
echo "Get task"
curl -X GET http://localhost:5000/task/test-003
echo "Got task"
echo "Get task"
curl -X GET http://localhost:5000/task/test-004
echo "Got task"

echo "Get stats"
curl -X GET http://localhost:5000/stats
echo "Got stats"

echo "Try acces directly the worker"
echo "test accessing directly the worker running alone (should fail):"
curl -v -X GET http://localhost:8080/task/test-003
echo "done"

