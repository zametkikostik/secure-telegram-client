"""Интеграции с внешними сервисами"""
from .kaspi import kaspi_client, kaspi_webhook
from .halyk import halyk_client, halyk_webhook
from .tax_service import tax_service, declaration_generator
from .kaspi_shop import kaspi_shop, kaspi_webhook as kaspi_shop_webhook

__all__ = [
    "kaspi_client",
    "kaspi_webhook",
    "halyk_client",
    "halyk_webhook",
    "tax_service",
    "declaration_generator",
    "kaspi_shop",
    "kaspi_shop_webhook"
]
