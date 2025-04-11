/**
 * Sensor Monitoring Visualization
 * JavaScript functions for creating charts and visualizations using Chart.js
 */

// Initialize charts when document is ready
document.addEventListener('DOMContentLoaded', function() {
    // Check if chart containers exist
    if (document.getElementById('temperatureChart')) {
      fetchTemperatureData();
    }
    
    if (document.getElementById('powerConsumptionChart')) {
      fetchPowerConsumptionData();
    }
    
    if (document.getElementById('sensorStatusChart')) {
      initializeSensorStatusChart();
    }
  });
  
  /**
   * Fetch temperature data from the API and create a line chart
   */
  function fetchTemperatureData() {
    // Get temperature sensor IDs
    const temperatureSensors = document.getElementById('temperatureChart').dataset.sensorIds;
    if (!temperatureSensors) return;
    
    const sensorIds = temperatureSensors.split(',');
    const endTime = Math.floor(Date.now() / 1000);
    const startTime = endTime - (24 * 60 * 60); // Last 24 hours
    
    fetch(`/api/proxy/visualizations/time-series?sensor_ids=${sensorIds.join(',')}&start_time=${startTime}&end_time=${endTime}&interval=hour`)
      .then(response => response.json())
      .then(data => {
        renderTemperatureChart(data);
      })
      .catch(error => console.error('Error fetching temperature data:', error));
  }
  
  /**
   * Render temperature line chart
   */
  function renderTemperatureChart(data) {
    const ctx = document.getElementById('temperatureChart').getContext('2d');
    
    const datasets = data.datasets.map(dataset => {
      return {
        label: dataset.sensor_name,
        data: dataset.data,
        borderColor: getColorForSensor(dataset.sensor_id),
        backgroundColor: 'transparent',
        tension: 0.1,
        pointRadius: 2,
        pointHoverRadius: 5
      };
    });
    
    const chart = new Chart(ctx, {
      type: 'line',
      data: {
        labels: data.labels,
        datasets: datasets
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Temperature Readings (Last 24 Hours)'
          },
          tooltip: {
            mode: 'index',
            intersect: false
          },
          legend: {
            position: 'bottom'
          }
        },
        scales: {
          x: {
            display: true,
            title: {
              display: true,
              text: 'Time'
            }
          },
          y: {
            display: true,
            title: {
              display: true,
              text: 'Temperature (°C)'
            }
          }
        }
      }
    });
  }
  
  /**
   * Fetch power consumption data and create a bar chart
   */
  function fetchPowerConsumptionData() {
    // Get power sensor IDs
    const powerSensors = document.getElementById('powerConsumptionChart').dataset.sensorIds;
    if (!powerSensors) return;
    
    const sensorIds = powerSensors.split(',');
    const endTime = Math.floor(Date.now() / 1000);
    const startTime = endTime - (24 * 60 * 60); // Last 24 hours
    
    fetch(`/api/proxy/visualizations/time-series?sensor_ids=${sensorIds.join(',')}&start_time=${startTime}&end_time=${endTime}&interval=hour`)
      .then(response => response.json())
      .then(data => {
        renderPowerConsumptionChart(data);
      })
      .catch(error => console.error('Error fetching power consumption data:', error));
  }
  
  /**
   * Render power consumption bar chart
   */
  function renderPowerConsumptionChart(data) {
    const ctx = document.getElementById('powerConsumptionChart').getContext('2d');
    
    const datasets = data.datasets.map(dataset => {
      return {
        label: dataset.sensor_name,
        data: dataset.data,
        backgroundColor: getColorForSensor(dataset.sensor_id, 0.7),
        borderColor: getColorForSensor(dataset.sensor_id),
        borderWidth: 1
      };
    });
    
    const chart = new Chart(ctx, {
      type: 'bar',
      data: {
        labels: data.labels,
        datasets: datasets
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Power Consumption (Last 24 Hours)'
          },
          tooltip: {
            mode: 'index',
            intersect: false
          },
          legend: {
            position: 'bottom'
          }
        },
        scales: {
          x: {
            display: true,
            title: {
              display: true,
              text: 'Time'
            }
          },
          y: {
            display: true,
            title: {
              display: true,
              text: 'Power (kW)'
            }
          }
        }
      }
    });
  }
  
  /**
   * Initialize sensor status doughnut chart
   */
  function initializeSensorStatusChart() {
    fetch('/api/proxy/status/health')
      .then(response => response.json())
      .then(data => {
        renderSensorStatusChart(data);
      })
      .catch(error => {
        console.error('Error fetching sensor status:', error);
        // Show mock data if API fails
        renderSensorStatusChart({
          healthy_count: 8,
          warning_count: 2,
          critical_count: 0
        });
      });
  }
  
  /**
   * Render sensor status doughnut chart
   */
  function renderSensorStatusChart(data) {
    const ctx = document.getElementById('sensorStatusChart').getContext('2d');
    
    const chart = new Chart(ctx, {
      type: 'doughnut',
      data: {
        labels: ['Healthy', 'Warning', 'Critical'],
        datasets: [{
          data: [data.healthy_count, data.warning_count, data.critical_count],
          backgroundColor: ['#28a745', '#ffc107', '#dc3545'],
          borderColor: ['#1e7e34', '#d39e00', '#bd2130'],
          borderWidth: 1
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Sensor Status'
          },
          legend: {
            position: 'bottom'
          },
          tooltip: {
            callbacks: {
              label: function(context) {
                const label = context.label || '';
                const value = context.raw || 0;
                const total = context.dataset.data.reduce((a, b) => a + b, 0);
                const percentage = Math.round((value / total) * 100);
                return `${label}: ${value} (${percentage}%)`;
              }
            }
          }
        },
        cutout: '70%'
      }
    });
  }
  
  /**
   * Generate a consistent color for a sensor based on its ID
   */
  function getColorForSensor(sensorId, alpha = 1) {
    // List of distinct colors
    const colors = [
      `rgba(255, 99, 132, ${alpha})`,    // Red
      `rgba(54, 162, 235, ${alpha})`,    // Blue
      `rgba(255, 206, 86, ${alpha})`,    // Yellow
      `rgba(75, 192, 192, ${alpha})`,    // Green
      `rgba(153, 102, 255, ${alpha})`,   // Purple
      `rgba(255, 159, 64, ${alpha})`,    // Orange
      `rgba(199, 199, 199, ${alpha})`,   // Gray
      `rgba(83, 102, 255, ${alpha})`,    // Indigo
      `rgba(255, 99, 255, ${alpha})`,    // Pink
      `rgba(0, 162, 151, ${alpha})`      // Teal
    ];
    
    // Use modulo to ensure we always get a valid index
    return colors[sensorId % colors.length];
  }
  
  /**
   * Update charts with real-time data
   */
  function refreshCharts() {
    if (document.getElementById('temperatureChart')) {
      fetchTemperatureData();
    }
    
    if (document.getElementById('powerConsumptionChart')) {
      fetchPowerConsumptionData();
    }
    
    if (document.getElementById('sensorStatusChart')) {
      initializeSensorStatusChart();
    }
  }
  
  // Set up auto-refresh for charts every 60 seconds
  setInterval(refreshCharts, 60000);