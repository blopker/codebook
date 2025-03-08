# On a sepaate line
class Foo # or at the end of the lne
  # can be inented
  def bar
  end
end

=begin
This is
comented out
=end

class Foo
end

=begin some_tag
this wors, too
=end

# frozen_string_lteral: true

var = 'helo'
var.frozen? # => true

# String interpolation with typos
user_message = "Hello #{user_name}, your acount has ben verrified"
system_notice = "#{total_count} itmes have ben proccessed"
alert = "#{error_count} erors ocurred during the opration"

# String with escape characters
escaped_message = "Please submitt your \"ressume\" for consideraton"

# Array of strings with typos
menu_items = ['Hambuger', 'Chesee Sandwitch', 'Frennch Fries', 'Chiken Nugetts']

# Hash with string keys and values containing typos
config = {
  'databse_host' => 'locallhost',
  'maximun_retres' => '5',
  'defalt_langauge' => 'Englsh'
}

# Multiline strings with %Q syntax
instructions = %Q{
  To resett your pasword:
  1. Clickk on the "Forgott Password" link
  2. Entter your email adress
  3. Folllow the instrucctions sent to your inbox
}

# String examples with typos in heredocs that could be checked with a spell checker
long_text = <<~TEXT
  This is a long peice of text with severel spelling erors.
  It would be verry usefull to run this through a spellchecker.
  The documantation should be clear and profesional.
TEXT

sql_comment = <<~SQL
  -- This querry selects all users who haven't confimed their email adresses
  -- It's importent to regulerly clean up unverified acounts
SQL

html_content = <<-HTML
  <div class="content">
    <h1>Wellcome to our webiste!</h1>
    <p>We're exited to anounce our new fetures.</p>
    <p>Please contect us if you encountar any isues.</p>
  </div>
HTML

# Method with string arguments containing typos
def send_notification(recipient, subject, body)
  # This method sends an email with potentialy misspelled content
  email = Email.new(
    to: recipient,
    subject: "URGENT: #{subject}",
    body: "Dear valued custommer,\n\n#{body}\n\nRegads,\nSuport Team"
  )
  email.send
end

# Strings in conditionals
if status == "complette" || status == "partialy_compleet"
  mark_as_finnished(item)
end

# JSON-like strings with typos
json_data = '{"username":"janedoe","prefferences":{"notifcations":true,"langauge":"en"}}'

# SQL queries in strings
query = "SELECT * FROM ussers WHERE last_loggin < DATE_SUB(NOW(), INTERVAL 30 DAY);"
