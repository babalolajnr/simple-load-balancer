import asyncio

# Configuration
LB_ADDRESS = "127.0.0.1"
LB_PORT = 8080
NUM_CLIENTS = 10  # How many simultaneous clients to simulate
CONNECTION_DELAY = 0.5  # Seconds to wait between starting new clients
WORK_DURATION = 2  # How long each client stays connected


async def simulate_client(client_id):
    """Simulates a single client connecting and sending data."""
    try:
        # 1. Open a connection to the Load Balancer
        reader, writer = await asyncio.open_connection(LB_ADDRESS, LB_PORT)
        print(f"👤 Client {client_id}: Connected to Load Balancer")

        # 2. Send a unique message
        message = f"Hello from Client {client_id}\n"
        writer.write(message.encode())
        await writer.drain()

        # 3. Hold the connection open to simulate 'work'
        # This is crucial for testing Least Connections!
        await asyncio.sleep(WORK_DURATION)

        # 4. Close the connection
        writer.close()
        await writer.wait_closed()
        print(f"👤 Client {client_id}: Disconnected")

    except Exception as e:
        print(f"❌ Client {client_id} Error: {e}")


async def main():
    print(f"🚀 Starting simulation with {NUM_CLIENTS} concurrent clients...")
    tasks = []

    for i in range(1, NUM_CLIENTS + 1):
        # We stagger the start slightly to see the LB logs clearly
        tasks.append(simulate_client(i))
        await asyncio.sleep(CONNECTION_DELAY)

    # Run all simulated clients concurrently
    await asyncio.gather(*tasks)
    print("✨ Simulation complete.")


if __name__ == "__main__":
    asyncio.run(main())
