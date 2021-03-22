using System;
using System.Drawing;
using System.Windows.Forms;
using System.Threading;
using Renci.SshNet;
using System.Net;
using System.Net.Sockets;
using System.Globalization;
using System.Net.NetworkInformation;
using System.Text.RegularExpressions;

namespace Server_Manager
{
    public partial class Main : Form
    {
        public Main()
        {
            InitializeComponent();
        }

        private void Settings_Click(object sender, EventArgs e)
        {
            Settings stgs = new Settings();
            stgs.Show();
        }

        private void Open_win_exporer_Click(object sender, EventArgs e)
        {
            var IP = Properties.Settings.Default.IP;
            Thread exp = new Thread(() =>
            {
                _ = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo()
                {
                    FileName = "\\\\" + IP + "\\",
                    UseShellExecute = true,
                    Verb = "open"
                });
            });
            exp.Start();
        }

        private bool Ping_IP()
        {
            var IP = Properties.Settings.Default.IP;
            Ping objping = new Ping();
            if (objping.Send(IP, 500).Status == IPStatus.Success) { return true; } else { return false; }
        }

        private void Main_Load(object sender, EventArgs e)
        {
            Auto_Ping.Start();

            Thread pingTest = new Thread(() =>
            {
                Status = Ping_IP();
                Action action = new Action(Update_Labels);
                BeginInvoke(action);
            });
            pingTest.Start();
        }

        bool Status = false;

        private void Ping_button_Click(object sender, EventArgs e)
        {
            Thread pingTest = new Thread(() =>
            {
                Status = Ping_IP();
                Action action = new Action(Update_Labels);
                BeginInvoke(action);
            });
            pingTest.Start();

            Ping_Button.Enabled = false;
            label3.Text = "CHECKING...";
            label3.ForeColor = Color.Black;
        }

        private void Update_Labels()
        {
            if (Status == true)
            {
                label3.Text = "ONLINE";
                label3.ForeColor = Color.Green;
                Power_ON.Enabled = false;
                Power_OFF.Enabled = true;
                Open_Win_Explorer.Enabled = true;
            }
            else
            {
                label3.Text = "OFFLINE";
                label3.ForeColor = Color.Red;
                Power_ON.Enabled = true;
                Power_OFF.Enabled = false;
                Open_Win_Explorer.Enabled = false;
            }
            Ping_Button.Enabled = true;
        }

        private void Update_Labels_Auto()
        {
            if (Status == true)
            {
                if (Power_ON.Text == "Booting...")
                {
                    Power_ON.Enabled = false;
                    Power_OFF.Enabled = true;
                    Open_Win_Explorer.Enabled = true;
                }
                label3.Text = "ONLINE";
                label3.ForeColor = Color.Green;
            }
            else
            {
                if (Power_OFF.Text == "Shutting Down...")
                {
                    Power_ON.Enabled = true;
                    Power_OFF.Enabled = false;
                    Open_Win_Explorer.Enabled = false;
                }
                label3.Text = "OFFLINE";
                label3.ForeColor = Color.Red;
            }
        }

        static long IPToInt(string addr)
        {
            return (long) (uint) IPAddress.NetworkToHostOrder(
                (int) IPAddress.Parse(addr).Address
            );
        }

        private static void WakeUp(string macAddress)
        {
            var IP = Properties.Settings.Default.BroadcastIP;
            string[] Tmp_IP = IP.Split('.');
            Array.Reverse(Tmp_IP);
            string Final_IP = string.Join(".", Tmp_IP);
            var Hex_IP = IPToInt(Final_IP);

            int Port = int.Parse(Properties.Settings.Default.Port);
            string Str_Port = Port.ToString("X");
            int Hex_Port = int.Parse(Str_Port , NumberStyles.HexNumber);

            MessageBox.Show(Hex_IP.ToString());

            WOLClass client = new WOLClass();
            client.Connect(new IPAddress(Hex_IP), Hex_Port);
            client.SetClientToBroadcastMode();

            int counter = 0;

            byte[] bytes = new byte[1024];

            for (int e = 0; e < 6; e++)
            {
                bytes[counter++] = 0xFF;
            }

            for (int e = 0; e < 16; e++)
            {
                int i = 0;

                for (int w = 0; w < 6; w++)
                {
                    bytes[counter++] = byte.Parse(macAddress.Substring(i, 2), NumberStyles.HexNumber);
                    i += 2;
                }
            }

            int returnedValue = client.Send(bytes, 1024);
        }

        public class WOLClass : UdpClient
        {
            public WOLClass()
                : base()
            {

            }

            public void SetClientToBroadcastMode()
            {
                if (this.Active)
                {
                    this.Client.SetSocketOption(SocketOptionLevel.Socket, SocketOptionName.Broadcast, 0);
                }
            }
        }

        private void Power_on_Click(object sender, EventArgs e)
        {
            var MAC = Regex.Replace(Properties.Settings.Default.MAC, @"[^0-9a-fA-F]+", "");

            if (MessageBox.Show("Are you sure?", "Power ON", MessageBoxButtons.YesNo, MessageBoxIcon.Question) == DialogResult.Yes)
            {
                Power_ON.Enabled = false;
                Power_ON.Text = "Booting...";
                WakeUp(MAC);
                Power_ON.Text = "Power ON";
            }
            else
            {
                Power_ON.Enabled = true;
                Power_ON.Text = "Power ON";
            }
        }

        private void Power_off_Click(object sender, EventArgs e)
        {
            if (MessageBox.Show("Are you sure?", "Power OFF", MessageBoxButtons.YesNo, MessageBoxIcon.Question) == DialogResult.Yes)
            {
                Power_OFF.Enabled = false;
                Power_OFF.Text = "Shutting Down...";
                var IP = Properties.Settings.Default.IP;
                var Username = Properties.Settings.Default.Username;
                var Password = Properties.Settings.Default.Password;
                var CMD = Properties.Settings.Default.ShutdownCMD;

                using (var client = new SshClient(IP, Username, Password))
                {
                    try
                    {
                        client.Connect();
                        client.RunCommand(CMD);
                        client.Disconnect();
                        Power_OFF.Enabled = true;
                        Power_OFF.Text = "Power OFF";
                    }
                    catch
                    {
                        MessageBox.Show("Unable to Power OFF - Connection Refused", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                        client.Dispose();
                        Power_OFF.Enabled = true;
                        Power_OFF.Text = "Power OFF";
                    }
                }
            }
            else
            {
                Power_OFF.Enabled = true;
                Power_OFF.Text = "Power OFF";
            }
        }

        private void Auto_Ping_Tick(object sender, EventArgs e)
        {
            Thread pingTest = new Thread(() =>
            {
                Status = Ping_IP();
                Action action = new Action(Update_Labels_Auto);
                BeginInvoke(action);
            });
            pingTest.Start();
        }
    }
}
