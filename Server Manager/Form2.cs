using System;
using System.Windows.Forms;

namespace Server_Manager
{
    public partial class Settings : Form
    {
        public Settings()
        {
            InitializeComponent();
        }

        private void Button1_Click(object sender, EventArgs e)
        {
            button1.Text = "Saved!";

            button1.Enabled = false;

             Properties.Settings.Default.IP = textBox1.Text;
             Properties.Settings.Default.Username = textBox6.Text;
             Properties.Settings.Default.Password = textBox5.Text;
             Properties.Settings.Default.ShutdownCMD = textBox8.Text;
             Properties.Settings.Default.MAC = textBox9.Text;
             Properties.Settings.Default.BroadcastIP = textBox7.Text;
             Properties.Settings.Default.Port = textBox2.Text;

            Properties.Settings.Default.Save();
            Properties.Settings.Default.Reload();

            checkBox1.Checked = false;
            checkBox4.Checked = false;
            checkBox5.Checked = false;
            checkBox6.Checked = false;
            checkBox8.Checked = false;
            textBox1.Enabled = false;
            textBox2.Enabled = false;
            textBox5.Enabled = false;
            textBox6.Enabled = false;
            textBox7.Enabled = false;
            textBox8.Enabled = false;
            textBox9.Enabled = false;

            System.Threading.Thread.Sleep(500);

            this.Hide();
            this.Dispose();
        }

        private void CheckBox1_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox1.Enabled == true)
            {
                textBox1.Enabled = false;
            }
            else
            {
                textBox1.Enabled = true;
            }

        }

        private void CheckBox2_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox2.Enabled == true)
            {
                textBox2.Enabled = false;
            }
            else
            {
                textBox2.Enabled = true;
            }
        }

        private void Settings_Load(object sender, EventArgs e)
        {
            textBox1.Text = Properties.Settings.Default.IP;
            textBox6.Text = Properties.Settings.Default.Username;
            textBox5.Text = Properties.Settings.Default.Password;
            textBox8.Text = Properties.Settings.Default.ShutdownCMD;
            textBox9.Text = Properties.Settings.Default.MAC;
            textBox7.Text = Properties.Settings.Default.BroadcastIP;
            textBox2.Text = Properties.Settings.Default.Port;
        }

        private void CheckBox4_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox6.Enabled == true)
            {
                textBox6.Enabled = false;
                textBox5.Enabled = false;
            }
            else
            {
                textBox6.Enabled = true;
                textBox5.Enabled = true;
            }
        }

        private void Button2_Click_1(object sender, EventArgs e)
        {
            MessageBox.Show("Created by Michał Czyż\n\nCopyright(c) 2021 MIT License\n\nVersion: 1.0.0.0", "About Server Manager", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }

        private void checkBox5_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox7.Enabled == true)
            {
                textBox7.Enabled = false;
            }
            else
            {
                textBox7.Enabled = true;
            }
        }

        private void checkBox6_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox8.Enabled == true)
            {
                textBox8.Enabled = false;
            }
            else
            {
                textBox8.Enabled = true;
            }
        }

        private void checkBox8_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox9.Enabled == true)
            {
                textBox9.Enabled = false;
            }
            else
            {
                textBox9.Enabled = true;
            }
        }
    }
}
