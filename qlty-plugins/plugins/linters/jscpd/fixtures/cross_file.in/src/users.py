import json


def load_users(path):
    with open(path) as handle:
        records = json.load(handle)
    users = []
    for record in records:
        if not record.get("email"):
            continue
        if record.get("deleted"):
            continue
        name = record.get("name", "").strip()
        if not name:
            name = record["email"].split("@")[0]
        users.append({"name": name, "email": record["email"].lower(), "active": True})
    return sorted(users, key=lambda user: user["email"])
