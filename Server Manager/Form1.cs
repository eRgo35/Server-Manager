using System;
using System.Drawing;
using System.Linq;
using System.Windows.Forms;
using System.Threading;
using Renci.SshNet;
using Renci.SshNet.Common;

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
            var OSIP = Properties.Settings.Default.OSIP;
            Thread exp = new Thread(() =>
            {
                _ = System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo()
                {
                    FileName = "\\\\" + OSIP + "\\",
                    UseShellExecute = true,
                    Verb = "open"
                });
            });
            exp.Start();
        }

        private bool checkNetCardIP()
        {
            var NetCardIP = Properties.Settings.Default.NetCardIP;

            System.Net.NetworkInformation.Ping objping = new System.Net.NetworkInformation.Ping();
            if (objping.Send(NetCardIP, 1000).Status == System.Net.NetworkInformation.IPStatus.Success)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        private bool checkOSIP()
        {
            var OSIP = Properties.Settings.Default.OSIP;

            System.Net.NetworkInformation.Ping objping = new System.Net.NetworkInformation.Ping();
            if (objping.Send(OSIP, 1000).Status == System.Net.NetworkInformation.IPStatus.Success)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        private void Main_Load(object sender, EventArgs e)
        {
            // timer1.Start();

            Thread pingTest = new Thread(() =>
            {
                NetCardStatus = checkNetCardIP();
                OSStatus = checkOSIP();
                Action action = new Action(SetColorsofLabels);
                BeginInvoke(action);
            });
            pingTest.Start();
        }

        bool NetCardStatus = false;
        bool OSStatus = false;

        private void Ping_button_Click(object sender, EventArgs e)
        {
            Thread pingTest = new Thread(() =>
            {
                NetCardStatus = checkNetCardIP();
                OSStatus = checkOSIP();
                Action action = new Action(SetColorsofLabels);
                BeginInvoke(action);
            });
            pingTest.Start();

            ping_button.Enabled = false;
            label3.Text = "CHECKING...";
            label3.ForeColor = Color.Black;
            label4.Text = "CHECKING...";
            label4.ForeColor = Color.Black;
        }

        private void SetColorsofLabels()
        {
            if (NetCardStatus == true)
            {
                label3.Text = "ONLINE";
                label3.ForeColor = Color.Green;
            }
            else
            {
                label3.Text = "OFFLINE";
                label3.ForeColor = Color.Red;
            }
            if (OSStatus == true)
            {
                label4.Text = "ONLINE";
                label4.ForeColor = Color.Green;
            }
            else
            {
                label4.Text = "OFFLINE";
                label4.ForeColor = Color.Red;
            }

            if (label3.Text == "OFFLINE" && label4.Text == "OFFLINE")
            {
                power_on.Enabled = false;
                power_off.Enabled = false;
                open_win_exporer.Enabled = false;
            }
            else if (label3.Text == "ONLINE" && label4.Text == "OFFLINE")
            {
                power_on.Enabled = true;
                power_off.Enabled = false;
                open_win_exporer.Enabled = false;
            }
            else if (label3.Text == "ONLINE" && label4.Text == "ONLINE")
            {
                power_on.Enabled = false;
                power_off.Enabled = true;
                open_win_exporer.Enabled = true;
            }
            ping_button.Enabled = true;
        }

        private void Power_on_Click(object sender, EventArgs e)
        {
            power_on.Enabled = false;
            power_on.Text = "Booting...";

            var clientip = Properties.Settings.Default.NetCardIP;
            var username = Properties.Settings.Default.NetCardUser;
            var password = Properties.Settings.Default.NetCardPass;
            var boot = Properties.Settings.Default.StartCmd;

            var authMethod = new KeyboardInteractiveAuthenticationMethod(username);
            authMethod.AuthenticationPrompt += (sender2, d) =>
            {
                d.Prompts.Single().Response = password;
            };

            using (var client = new SshClient(clientip, username, authMethod.ToString()))
            {
                try
                {
                    client.HostKeyReceived += delegate (object s, HostKeyEventArgs c)
                    {
                        c.CanTrust = true;
                    };
                    client.Connect();
                    client.RunCommand(boot).Execute();
                    MessageBox.Show("Test");
                    client.Disconnect();
                }
                catch (Exception ex)
                {
                    MessageBox.Show(ex.ToString(), "Error! Unable to Power ON!", MessageBoxButtons.OK, MessageBoxIcon.Error);
                    client.Dispose();
                    power_on.Text = "Power ON";
                    power_on.Enabled = true;
                }
            }
        }

        private void Power_off_Click(object sender, EventArgs e)
        {
            power_off.Enabled = false;
            power_off.Text = "Shutting Down...";
            var clientip = Properties.Settings.Default.OSIP;
            var username = Properties.Settings.Default.OSUser;
            var password = Properties.Settings.Default.OSPass;
            var shutdown = Properties.Settings.Default.ShutDwnCmd;

            using (var client = new SshClient(clientip, username, password))
            {
                try
                {
                    client.Connect();
                    client.RunCommand(shutdown);
                    client.Disconnect();
                }
                catch
                {
                    MessageBox.Show("Unable to Power OFF - Connection Refused", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                    client.Dispose();
                    power_off.Text = "Power OFF";
                }
            }
        }

        private void Timer1_Tick(object sender, EventArgs e)
        {
            ping_button.PerformClick();
        }
    }
}
