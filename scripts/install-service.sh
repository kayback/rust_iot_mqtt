#!/bin/bash
set -e

echo "Install IoT Ingestor as systemd service"
echo "=============================================="
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

SERVICE_NAME="iot-ingestor"
INSTALL_DIR="/opt/iot-ingestor"
SERVICE_USER="iot"
SERVICE_GROUP="iot"
BINARY_PATH="target/release/ingestor"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found. Please build first:"
    echo "   cargo build --release"
    exit 1
fi

if ! id "$SERVICE_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" "$SERVICE_USER"
fi

mkdir -p "$INSTALL_DIR/bin"
mkdir -p "$INSTALL_DIR/migrations"
mkdir -p /var/log/iot-ingestor

cp "$BINARY_PATH" "$INSTALL_DIR/bin/ingestor"
cp -r ingestor/migrations/* "$INSTALL_DIR/migrations/"
chmod +x "$INSTALL_DIR/bin/ingestor"

chmod +x scripts/start-dependencies.sh

chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"
chown -R "$SERVICE_USER:$SERVICE_GROUP" /var/log/iot-ingestor

cp ingestor/systemd/iot-ingestor.service /etc/systemd/system/
systemctl daemon-reload

echo "Installation complete!"
echo ""
echo "Next steps:"
echo ""
echo "1. Configure the service:"
echo "   sudo nano /etc/systemd/system/iot-ingestor.service"
echo ""
echo "2. Run start-dependencies.sh:"
echo "   sudo ./scripts/start-dependencies.sh "
echo ""
echo "3. Enable service to start on boot:"
echo "   sudo systemctl enable $SERVICE_NAME"
echo ""
echo "4. Start the service:"
echo "   sudo systemctl start $SERVICE_NAME"
echo ""
echo "5. Check status:"
echo "   sudo systemctl status $SERVICE_NAME"
echo ""
echo "6. View logs:"
echo "   sudo journalctl -u $SERVICE_NAME -f"
echo ""

