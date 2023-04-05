import requests

resp = requests.post("http://localhost:8000/appendentries",  json={
    "term": 43,
    "leader_id": 5
})

print(resp.status_code, resp.text)
