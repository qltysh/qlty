import os
import sqlite3

def insecure_database():
    user_input = input("Enter username: ")
    query = f"SELECT * FROM users WHERE name = '{user_input}'"
    # TODO: This is vulnerable to SQL injection
    conn = sqlite3.connect("users.db")
    cursor = conn.cursor()
    cursor.execute(query)
    print(cursor.fetchall())
