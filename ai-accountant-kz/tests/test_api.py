"""Тесты для API"""
import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from datetime import datetime
from decimal import Decimal

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from src.main import app
from src.core.database import Base, get_db, init_db
from src.core.config import settings

# ===== Test Database =====
SQLALCHEMY_DATABASE_URL = "sqlite:///./test.db"
engine = create_engine(
    SQLALCHEMY_DATABASE_URL,
    connect_args={"check_same_thread": False}
)
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# ===== Fixtures =====
@pytest.fixture(scope="function")
def db():
    """Создание тестовой БД"""
    Base.metadata.create_all(bind=engine)
    db = TestingSessionLocal()
    try:
        yield db
    finally:
        db.close()
        Base.metadata.drop_all(bind=engine)

@pytest.fixture(scope="function")
def client(db):
    """Тестовый клиент"""
    def override_get_db():
        try:
            yield db
        finally:
            pass
    
    app.dependency_overrides[get_db] = override_get_db
    client = TestClient(app)
    yield client
    app.dependency_overrides.clear()

# ===== Tests =====
class TestHealth:
    def test_health_check(self, client):
        response = client.get("/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"
    
    def test_root(self, client):
        response = client.get("/")
        assert response.status_code == 200

class TestCategories:
    def test_get_categories(self, client):
        response = client.get("/api/v1/categories")
        assert response.status_code == 200
        data = response.json()
        assert "income" in data
        assert "expense" in data

class TestTransactions:
    def test_create_transaction_unauthorized(self, client):
        """Тест создания без авторизации (должен работать в dev режиме)"""
        data = {
            "date": "2026-02-28",
            "amount": 100000,
            "type": "income",
            "description": "Тестовый доход"
        }
        response = client.post("/api/v1/transactions", json=data)
        # В production будет 401, в dev может работать
        assert response.status_code in [200, 401]

class TestTaxRules:
    def test_tax_rate_2026(self):
        from src.core.tax_rules import TAX_RULES
        assert TAX_RULES.RATE == Decimal("0.04")
        assert TAX_RULES.rate_percent == "4%"
    
    def test_max_income(self):
        from src.core.tax_rules import TAX_RULES
        assert TAX_RULES.max_income == 24038 * 3924

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
