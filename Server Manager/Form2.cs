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

        private void GroupBox2_Enter(object sender, EventArgs e)
        {

        }

        private void Button1_Click(object sender, EventArgs e)
        {
            button1.Text = "Saved!";

            button1.Enabled = false;

            Properties.Settings.Default.NetCardIP = textBox1.Text;
            Properties.Settings.Default.OSIP = textBox2.Text;
            Properties.Settings.Default.NetCardUser = textBox3.Text;
            Properties.Settings.Default.NetCardPass = textBox4.Text;
            Properties.Settings.Default.OSUser = textBox6.Text;
            Properties.Settings.Default.OSPass = textBox5.Text;
            Properties.Settings.Default.StartCmd = textBox7.Text;
            Properties.Settings.Default.ShutDwnCmd = textBox8.Text;

            Properties.Settings.Default.Save();
            Properties.Settings.Default.Reload();

            checkBox1.Checked = false;
            checkBox2.Checked = false;
            checkBox3.Checked = false;
            checkBox4.Checked = false;

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
            textBox1.Text = Properties.Settings.Default.NetCardIP;
            textBox2.Text = Properties.Settings.Default.OSIP;
            textBox3.Text = Properties.Settings.Default.NetCardUser;
            textBox4.Text = Properties.Settings.Default.NetCardPass;
            textBox6.Text = Properties.Settings.Default.OSUser;
            textBox5.Text = Properties.Settings.Default.OSPass;
            textBox7.Text = Properties.Settings.Default.StartCmd;
            textBox8.Text = Properties.Settings.Default.ShutDwnCmd;
        }

        private void CheckBox3_CheckedChanged(object sender, EventArgs e)
        {
            if (textBox3.Enabled == true)
            {
                textBox3.Enabled = false;
                textBox4.Enabled = false;
            }
            else
            {
                textBox3.Enabled = true;
                textBox4.Enabled = true;
            }
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

        private void TextBox3_TextChanged(object sender, EventArgs e)
        {

        }

        private void Button2_Click_1(object sender, EventArgs e)
        {
            MessageBox.Show("Created by Michał Czyż\n\nCopyright(C) 2020 All Rights Reserved\n\nVersion: 1.0", "About", MessageBoxButtons.OK, MessageBoxIcon.Information);
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
    }
}
